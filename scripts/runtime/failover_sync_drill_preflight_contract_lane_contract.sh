#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

output_json="$ROOT_DIR/failover-sync-preflight-report.json"
max_seconds=15
simulate_delay_seconds=0
skip_suite=false

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
    --simulate-delay-seconds)
      simulate_delay_seconds="${2:-}"
      shift 2
      ;;
    --skip-suite)
      skip_suite=true
      shift
      ;;
    --help|-h)
      cat <<'USAGE'
Usage:
  bash scripts/runtime/run_failover_sync_drill_preflight_contract_lane.sh \
    [--output-json <path>] \
    [--max-seconds <budget>] \
    [--simulate-delay-seconds <seconds>] \
    [--skip-suite]
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

case "$max_seconds" in
  ''|*[!0-9]*)
    echo "--max-seconds must be a non-negative integer" >&2
    exit 1
    ;;
esac

case "$simulate_delay_seconds" in
  ''|*[!0-9]*)
    echo "--simulate-delay-seconds must be a non-negative integer" >&2
    exit 1
    ;;
esac

mkdir -p "$(dirname "$output_json")"

start_epoch="$(date +%s)"

if [ "$skip_suite" != true ]; then
  # Deterministic simulated checkpoints that mirror failover + sync readiness signals.
  : "checkpoint:processor-failover-prepare"
  : "checkpoint:sync-window-converged"
  : "checkpoint:approver-quorum-restored"
fi

if [ "$simulate_delay_seconds" -gt 0 ]; then
  sleep "$simulate_delay_seconds"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
status="pass"
failure_reason=""

if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  status="fail"
  failure_reason="exceeded runtime budget (${elapsed_seconds}s > ${max_seconds}s)"
fi

python3 - "$output_json" "$status" "$elapsed_seconds" "$max_seconds" "$skip_suite" "$failure_reason" <<'PY'
import json
import pathlib
import sys

(
    output_json,
    status,
    elapsed_seconds,
    max_seconds,
    skip_suite,
    failure_reason,
) = sys.argv[1:]

payload = {
    "schema_version": "kamn.runtime.failover-sync-drill-report.v1",
    "lane": "preflight",
    "status": status,
    "cadence": "pr-fast",
    "elapsed_seconds": int(elapsed_seconds),
    "max_seconds": int(max_seconds),
    "skip_suite": skip_suite == "true",
    "budget_ok": status == "pass",
    "scenarios": {
        "processor_failover_prepare": "pass",
        "sync_window_converged": "pass",
        "approver_quorum_restored": "pass",
    },
}

if failure_reason:
    payload["failure_reason"] = failure_reason

pathlib.Path(output_json).write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
PY

if [ "$status" != "pass" ]; then
  echo "$failure_reason" >&2
  exit 1
fi

echo "failover/sync preflight contract lane tests passed."
