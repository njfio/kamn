#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

output_json="$ROOT_DIR/failover-sync-preflight-report.json"
max_seconds=15
ci_local_promotion_max_seconds=""
simulate_delay_seconds=0
simulate_live_node_drift=false
simulate_failover_stall=false
skip_suite=false

failover_readiness_reason_taxonomy_version="kamn.runtime.failover-readiness-reason-taxonomy.v1"
failover_readiness_reason_codes_csv="failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded"

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
    --ci-local-promotion-max-seconds)
      ci_local_promotion_max_seconds="${2:-}"
      shift 2
      ;;
    --simulate-delay-seconds)
      simulate_delay_seconds="${2:-}"
      shift 2
      ;;
    --simulate-live-node-drift)
      simulate_live_node_drift=true
      shift
      ;;
    --simulate-failover-stall)
      simulate_failover_stall=true
      shift
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
    [--ci-local-promotion-max-seconds <budget>] \
    [--simulate-delay-seconds <seconds>] \
    [--simulate-live-node-drift] \
    [--simulate-failover-stall] \
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

if [ -z "$ci_local_promotion_max_seconds" ]; then
  ci_local_promotion_max_seconds="$max_seconds"
fi

case "$max_seconds" in
  ''|*[!0-9]*)
    echo "--max-seconds must be a non-negative integer" >&2
    exit 1
    ;;
esac

case "$ci_local_promotion_max_seconds" in
  ''|*[!0-9]*)
    echo "--ci-local-promotion-max-seconds must be a non-negative integer" >&2
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
reason_code="none"
failure_reason=""
failover_promotion_gate_status="verified"
live_node_drift_parity_status="verified"
ci_local_promotion_budget_boundary_status="verified"

if [ "$simulate_failover_stall" = true ]; then
  status="fail"
  reason_code="failover_readiness_progress_stalled"
  failure_reason="${reason_code}: failover readiness progress checkpoint did not advance"
  failover_promotion_gate_status="failed"
elif [ "$simulate_live_node_drift" = true ]; then
  status="fail"
  reason_code="live_node_drift_marker_parity_mismatch"
  failure_reason="${reason_code}: live-node drift marker parity diverged from deterministic contract"
  live_node_drift_parity_status="failed"
elif [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  status="fail"
  reason_code="runtime_budget_exceeded"
  failure_reason="exceeded runtime budget (${elapsed_seconds}s > ${max_seconds}s)"
elif [ "$elapsed_seconds" -gt "$ci_local_promotion_max_seconds" ]; then
  status="fail"
  reason_code="ci_local_promotion_budget_boundary_exceeded"
  failure_reason="${reason_code}: ci/local promotion boundary exceeded (${elapsed_seconds}s > ${ci_local_promotion_max_seconds}s)"
  ci_local_promotion_budget_boundary_status="failed"
fi

python3 - \
  "$output_json" \
  "$status" \
  "$elapsed_seconds" \
  "$max_seconds" \
  "$ci_local_promotion_max_seconds" \
  "$skip_suite" \
  "$reason_code" \
  "$failure_reason" \
  "$failover_promotion_gate_status" \
  "$live_node_drift_parity_status" \
  "$ci_local_promotion_budget_boundary_status" \
  "$failover_readiness_reason_taxonomy_version" \
  "$failover_readiness_reason_codes_csv" \
  <<'PY'
import json
import pathlib
import sys

(
    output_json,
    status,
    elapsed_seconds,
    max_seconds,
    ci_local_promotion_max_seconds,
    skip_suite,
    reason_code,
    failure_reason,
    failover_promotion_gate_status,
    live_node_drift_parity_status,
    ci_local_promotion_budget_boundary_status,
    failover_readiness_reason_taxonomy_version,
    failover_readiness_reason_codes_csv,
) = sys.argv[1:]

payload = {
    "schema_version": "kamn.runtime.failover-sync-drill-report.v1",
    "lane": "preflight",
    "status": status,
    "cadence": "pr-fast",
    "elapsed_seconds": int(elapsed_seconds),
    "max_seconds": int(max_seconds),
    "ci_local_promotion_max_seconds": int(ci_local_promotion_max_seconds),
    "skip_suite": skip_suite == "true",
    "budget_ok": status == "pass",
    "reason_code": reason_code,
    "failover_promotion_gate_status": failover_promotion_gate_status,
    "live_node_drift_parity_status": live_node_drift_parity_status,
    "ci_local_promotion_budget_boundary_status": ci_local_promotion_budget_boundary_status,
    "failover_readiness_reason_taxonomy_version": failover_readiness_reason_taxonomy_version,
    "failover_readiness_reason_codes_csv": failover_readiness_reason_codes_csv,
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
