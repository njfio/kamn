#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

output_json=""
max_seconds=120

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

prebuild_timeout_seconds="$max_seconds"
if [ "$prebuild_timeout_seconds" -lt 1 ]; then
  prebuild_timeout_seconds=1
fi

set +e
timeout "$prebuild_timeout_seconds" cargo build --quiet -p kamn-node --bin kamn-node
prebuild_exit=$?
set -e
if [ "$prebuild_exit" -ne 0 ]; then
  if [ "$prebuild_exit" -eq 124 ]; then
    echo "async runtime live validation prebuild timed out" >&2
  else
    echo "async runtime live validation prebuild failed" >&2
  fi
  exit "$prebuild_exit"
fi

node_binary="$ROOT_DIR/target/debug/kamn-node"
if [ ! -x "$node_binary" ]; then
  echo "expected prebuilt kamn-node binary at $node_binary" >&2
  exit 1
fi

command_timeout_seconds="$max_seconds"
if [ "$command_timeout_seconds" -lt 1 ]; then
  command_timeout_seconds=1
fi

run_with_timeout() {
  local output_file="$1"
  local timeout_seconds="$2"
  shift 2
  python3 - "$output_file" "$timeout_seconds" "$ROOT_DIR" "$@" <<'PY'
import pathlib
import subprocess
import sys

output_file = pathlib.Path(sys.argv[1])
timeout_seconds = int(sys.argv[2])
root_dir = sys.argv[3]
command = sys.argv[4:]

try:
    completed = subprocess.run(
        command,
        cwd=root_dir,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=timeout_seconds,
        check=False,
    )
except subprocess.TimeoutExpired as error:
    output = error.stdout or ""
    output_file.write_text(
        f"{output}\ncommand timed out after {timeout_seconds}s\n",
        encoding="utf-8",
    )
    raise SystemExit(124)

output_file.write_text(completed.stdout, encoding="utf-8")
raise SystemExit(completed.returncode)
PY
}

start_epoch="$(date +%s)"

success_stdout="$TMP_DIR/async-runtime-success.out"
set +e
run_with_timeout "$success_stdout" "$command_timeout_seconds" "$node_binary" \
  --role processor \
  --chain-id kamn-devnet \
  --chain-version v0.1.0 \
  --runtime-mode daemon \
  --daemon-max-ticks 3 \
  --daemon-tick-interval-ms 1 \
  --daemon-shutdown-signal-tick 1 \
  --daemon-shutdown-drain-ticks 1 \
  --daemon-shutdown-timeout-ticks 4 \
  --output json \
  --diagnostics basic
success_code=$?
set -e
if [ "$success_code" -ne 0 ]; then
  cat "$success_stdout" >&2
  echo "expected async runtime daemon execution path to succeed" >&2
  exit 1
fi

if ! grep -q '"runtime_mode":"daemon"' "$success_stdout"; then
  cat "$success_stdout" >&2
  echo "expected daemon runtime mode marker in async runtime output" >&2
  exit 1
fi

if ! grep -q '"daemon_completion_reason":"' "$success_stdout"; then
  cat "$success_stdout" >&2
  echo "expected daemon completion marker in async runtime output" >&2
  exit 1
fi

failure_stdout_file="$TMP_DIR/async-runtime-failure.out"
set +e
run_with_timeout "$failure_stdout_file" "$command_timeout_seconds" "$node_binary" \
  --role processor \
  --chain-id kamn-devnet \
  --chain-version v0.1.0 \
  --runtime-mode invalid \
  --output json
failure_code=$?
set -e
failure_stdout="$(cat "$failure_stdout_file")"
if [ "$failure_code" -eq 0 ]; then
  echo "expected invalid runtime-mode drill to fail closed" >&2
  exit 1
fi

if ! printf '%s\n' "$failure_stdout" | grep -qi 'invalid runtime mode'; then
  printf '%s\n' "$failure_stdout" >&2
  echo "expected invalid runtime mode reason marker in fail-closed drill" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "async runtime live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/async-runtime-live-validation-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.runtime.async-runtime-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "runtime_entrypoint": "tokio-main",
  "success_case_status": "verified",
  "failure_case_status": "verified",
  "failure_reason_marker": "invalid-runtime-mode",
  "elapsed_seconds": $elapsed_seconds
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "runtime_entrypoint=tokio-main"
echo "success_case_status=verified"
echo "failure_case_status=verified"
