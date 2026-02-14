#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

output_json=""
max_seconds=180

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
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"
test_output_file="$TMP_DIR/continuous-runtime-contract.out"
set +e
(
  cd "$ROOT_DIR"
  cargo test -p kamn-node -- \
    rejects_kolme_live_continuous_mode_without_tick_interval \
    rejects_kolme_live_continuous_mode_without_max_ticks \
    functional_runtime_kolme_live_continuous_mode_executes_multiple_cycles
) >"$test_output_file" 2>&1
test_code=$?
set -e

if [ "$test_code" -ne 0 ]; then
  cat "$test_output_file" >&2
  echo "continuous runtime commit contract lane failed" >&2
  exit 1
fi

if ! grep -q '3 passed; 0 failed' "$test_output_file"; then
  cat "$test_output_file" >&2
  echo "expected continuous runtime contract pass-count marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "continuous runtime commit contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/continuous-runtime-commit-contract-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.kolme.continuous-runtime-commit.contract.v1",
  "status": "pass",
  "final_decision": "GO",
  "continuous_mode_status": "verified",
  "finality_recovery_status": "verified",
  "fail_closed_guard_status": "verified",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "continuous_mode_status=verified"
echo "finality_recovery_status=verified"
echo "fail_closed_guard_status=verified"
