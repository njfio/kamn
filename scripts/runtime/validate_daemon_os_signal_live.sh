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

start_epoch="$(date +%s)"

pushd "$ROOT_DIR" >/dev/null
cargo build --quiet -p kamn-node
NODE_BIN="$ROOT_DIR/target/debug/kamn-node"
popd >/dev/null

if [ ! -x "$NODE_BIN" ]; then
  echo "expected built kamn-node binary to be executable" >&2
  exit 1
fi

signal_stdout="$TMP_DIR/daemon-os-signal.out"
"$NODE_BIN" \
  --role processor \
  --chain-id kamn-devnet \
  --chain-version v0.1.0 \
  --runtime-mode daemon \
  --daemon-max-ticks 200 \
  --daemon-tick-interval-ms 5 \
  --daemon-shutdown-os-signals \
  --daemon-shutdown-drain-ticks 2 \
  --daemon-shutdown-timeout-ticks 8 \
  --output json >"$signal_stdout" 2>&1 &
node_pid=$!

started=0
for _ in $(seq 1 50); do
  if grep -q 'node.runtime.daemon.execute.start' "$signal_stdout"; then
    started=1
    break
  fi
  if ! kill -0 "$node_pid" 2>/dev/null; then
    break
  fi
  sleep 0.02
done

if [ "$started" -ne 1 ]; then
  cat "$signal_stdout" >&2
  echo "expected daemon process to emit startup marker before signal drill" >&2
  kill -KILL "$node_pid" 2>/dev/null || true
  wait "$node_pid" 2>/dev/null || true
  exit 1
fi

kill -TERM "$node_pid"
set +e
wait "$node_pid"
node_exit_code=$?
set -e
if [ "$node_exit_code" -ne 0 ]; then
  cat "$signal_stdout" >&2
  echo "expected daemon os signal drill process to exit cleanly" >&2
  exit 1
fi

if ! grep -q '"runtime_mode":"daemon"' "$signal_stdout"; then
  cat "$signal_stdout" >&2
  echo "expected daemon runtime mode marker in os signal drill output" >&2
  exit 1
fi

completion_reason_marker="$(grep -o '"daemon_completion_reason":"[^"]*"' "$signal_stdout" | head -n 1 | cut -d '"' -f 4)"
if [ -z "$completion_reason_marker" ]; then
  cat "$signal_stdout" >&2
  echo "expected daemon completion reason marker in os signal drill output" >&2
  exit 1
fi

case "$completion_reason_marker" in
  graceful-shutdown:*|graceful-shutdown-timeout:*)
    ;;
  *)
    cat "$signal_stdout" >&2
    echo "expected graceful shutdown completion reason from os signal drill" >&2
    exit 1
    ;;
esac

set +e
failure_stdout="$($NODE_BIN \
  --role processor \
  --chain-id kamn-devnet \
  --chain-version v0.1.0 \
  --runtime-mode invalid \
  --output json 2>&1)"
failure_code=$?
set -e
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
  echo "daemon os signal live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/daemon-os-signal-live-validation-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.runtime.daemon-os-signal-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "os_signal_shutdown_status": "verified",
  "failure_case_status": "verified",
  "completion_reason_marker": "$completion_reason_marker",
  "failure_reason_marker": "invalid-runtime-mode",
  "elapsed_seconds": $elapsed_seconds
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "os_signal_shutdown_status=verified"
echo "failure_case_status=verified"
