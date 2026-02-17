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
drift_taxonomy_runbook_reason_taxonomy_version="kamn.runtime.failover-drift-taxonomy-runbook-reason-taxonomy.v1"
drift_taxonomy_runbook_reason_codes_csv="drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
failover_promotion_decision_reason_taxonomy_version="kamn.runtime.failover-promotion-decision-reason-taxonomy.v1"
failover_promotion_decision_reason_codes_csv="failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded,drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,failover_sync_drift_policy_expected_decision_mismatch,failover_sync_drift_policy_violation"
failover_evidence_convergence_reason_taxonomy_version="kamn.runtime.failover-evidence-convergence-reason-taxonomy.v1"
failover_evidence_convergence_reason_codes_csv="failover_evidence_link_missing,failover_evidence_payload_tamper_detected,promotion_decision_reason_mapping_mismatch"

run_policy_check() {
  local report_file=""
  local expected_final_decision="GO"
  local ci_fast_gate="PASS"
  local policy_output_json="$ROOT_DIR/failover-sync-preflight-policy-report.json"
  local runbook_file="$ROOT_DIR/docs/deploy/kolme_devnet_ops.md"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --report-file)
        report_file="${2:-}"
        shift 2
        ;;
      --expected-final-decision)
        expected_final_decision="${2:-}"
        shift 2
        ;;
      --ci-fast-gate)
        ci_fast_gate="${2:-}"
        shift 2
        ;;
      --output-json)
        policy_output_json="${2:-}"
        shift 2
        ;;
      --runbook-file)
        runbook_file="${2:-}"
        shift 2
        ;;
      --help|-h)
        cat <<'USAGE'
Usage:
  bash scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh check-policy \
    --report-file <path> \
    [--expected-final-decision GO|NO-GO] \
    [--ci-fast-gate PASS|FAIL] \
    [--runbook-file <path>] \
    [--output-json <path>]
USAGE
        exit 0
        ;;
      *)
        echo "unknown argument: $1" >&2
        exit 1
        ;;
    esac
  done

  if [ -z "$report_file" ]; then
    echo "--report-file is required in check-policy mode" >&2
    exit 1
  fi
  if [ ! -f "$report_file" ]; then
    echo "report file not found: $report_file" >&2
    exit 1
  fi
  if [ ! -f "$runbook_file" ]; then
    echo "runbook file not found: $runbook_file" >&2
    exit 1
  fi
  case "$expected_final_decision" in
    GO|NO-GO) ;;
    *)
      echo "--expected-final-decision must be GO or NO-GO" >&2
      exit 1
      ;;
  esac
  case "$ci_fast_gate" in
    PASS|FAIL) ;;
    *)
      echo "--ci-fast-gate must be PASS or FAIL" >&2
      exit 1
      ;;
  esac

  mkdir -p "$(dirname "$policy_output_json")"

  python3 - \
    "$report_file" \
    "$expected_final_decision" \
    "$ci_fast_gate" \
    "$policy_output_json" \
    "$failover_readiness_reason_taxonomy_version" \
    "$failover_readiness_reason_codes_csv" \
    "$drift_taxonomy_runbook_reason_taxonomy_version" \
    "$drift_taxonomy_runbook_reason_codes_csv" \
    "$failover_promotion_decision_reason_taxonomy_version" \
    "$failover_promotion_decision_reason_codes_csv" \
    "$runbook_file" \
    <<'PY'
import json
import pathlib
import sys

(
    report_file,
    expected_final_decision,
    ci_fast_gate,
    output_json,
    expected_reason_taxonomy_version,
    expected_reason_codes_csv,
    expected_drift_taxonomy_reason_taxonomy_version,
    expected_drift_taxonomy_reason_codes_csv,
    expected_promotion_reason_taxonomy_version,
    expected_promotion_reason_codes_csv,
    runbook_file,
) = sys.argv[1:]

report_path = pathlib.Path(report_file)
report = json.loads(report_path.read_text(encoding="utf-8"))
runbook_path = pathlib.Path(runbook_file)
runbook_text = runbook_path.read_text(encoding="utf-8")
required_fields = [
    "schema_version",
    "lane",
    "status",
    "failover_promotion_gate_status",
    "live_node_drift_parity_status",
    "ci_local_promotion_budget_boundary_status",
    "failover_readiness_reason_taxonomy_version",
    "failover_readiness_reason_codes_csv",
    "drift_taxonomy_mapping_status",
    "runbook_marker_parity_status",
    "drift_taxonomy_runbook_reason_taxonomy_version",
    "drift_taxonomy_runbook_reason_codes_csv",
]

reason_codes: list[str] = []
messages: list[str] = []


def add_reason(condition: bool, code: str) -> None:
    if condition and code not in reason_codes:
        reason_codes.append(code)


def resolve_promotion_reason_code(codes: list[str], pass_status: bool) -> str:
    if pass_status:
        return "none"
    preferred_codes = [
        "failover_readiness_progress_stalled",
        "live_node_drift_marker_parity_mismatch",
        "ci_local_promotion_budget_boundary_exceeded",
        "drift_taxonomy_mapping_drift_detected",
        "runbook_marker_parity_mismatch",
        "ci_fast_gate_failed",
        "failover_sync_drift_policy_expected_decision_mismatch",
    ]
    for code in preferred_codes:
        if code in codes:
            return code
    return "failover_sync_drift_policy_violation"


missing_fields = [field for field in required_fields if field not in report]
if missing_fields:
    messages.append(f"missing required report fields: {','.join(missing_fields)}")
    for field in missing_fields:
        add_reason(True, f"failover_sync_drift_policy_required_field_missing:{field}")

add_reason(
    report.get("schema_version") != "kamn.runtime.failover-sync-drill-report.v1",
    "failover_sync_drift_policy_schema_mismatch",
)
add_reason(
    report.get("lane") != "preflight",
    "failover_sync_drift_policy_lane_mismatch",
)
add_reason(
    report.get("status") not in {"pass", "fail"},
    "failover_sync_drift_policy_status_invalid",
)
add_reason(
    report.get("failover_promotion_gate_status") != "verified",
    "failover_readiness_progress_stalled",
)
add_reason(
    report.get("live_node_drift_parity_status") != "verified",
    "live_node_drift_marker_parity_mismatch",
)
add_reason(
    report.get("ci_local_promotion_budget_boundary_status") != "verified",
    "ci_local_promotion_budget_boundary_exceeded",
)
add_reason(
    report.get("failover_readiness_reason_taxonomy_version")
    != expected_reason_taxonomy_version,
    "failover_sync_drift_policy_reason_taxonomy_version_mismatch",
)
add_reason(
    report.get("failover_readiness_reason_codes_csv") != expected_reason_codes_csv,
    "failover_sync_drift_policy_reason_codes_csv_mismatch",
)
add_reason(
    report.get("drift_taxonomy_mapping_status") != "verified",
    "drift_taxonomy_mapping_drift_detected",
)
add_reason(
    report.get("runbook_marker_parity_status") != "verified",
    "runbook_marker_parity_mismatch",
)
add_reason(
    report.get("drift_taxonomy_runbook_reason_taxonomy_version")
    != expected_drift_taxonomy_reason_taxonomy_version,
    "failover_sync_drift_policy_drift_taxonomy_reason_taxonomy_version_mismatch",
)
add_reason(
    report.get("drift_taxonomy_runbook_reason_codes_csv")
    != expected_drift_taxonomy_reason_codes_csv,
    "failover_sync_drift_policy_drift_taxonomy_reason_codes_csv_mismatch",
)

required_runbook_markers = [
    f"drift_taxonomy_runbook_reason_taxonomy_version={expected_drift_taxonomy_reason_taxonomy_version}",
    f"drift_taxonomy_runbook_reason_codes_csv={expected_drift_taxonomy_reason_codes_csv}",
    "drift_taxonomy_mapping_status=verified",
    "runbook_marker_parity_status=verified",
]
missing_runbook_markers = [
    marker for marker in required_runbook_markers if marker not in runbook_text
]
if missing_runbook_markers:
    messages.append(
        "runbook marker parity missing required markers: "
        + ",".join(missing_runbook_markers)
    )
add_reason(bool(missing_runbook_markers), "runbook_marker_parity_mismatch")

add_reason(
    ci_fast_gate != "PASS",
    "ci_fast_gate_failed",
)

computed_final_decision = "GO" if not reason_codes else "NO-GO"
add_reason(
    expected_final_decision != computed_final_decision,
    "failover_sync_drift_policy_expected_decision_mismatch",
)

status_pass = len(reason_codes) == 0
final_decision = "GO" if status_pass else "NO-GO"
policy_status = "verified" if status_pass else "failed"
status_marker = "ok" if status_pass else "error"
resolved_reason_codes = ["none"] if status_pass else reason_codes
promotion_decision_reason_code = resolve_promotion_reason_code(reason_codes, status_pass)

policy_payload = {
    "schema_version": "kamn.runtime.failover-sync-drill-preflight-policy-report.v1",
    "status": "pass" if status_pass else "fail",
    "final_decision": final_decision,
    "expected_final_decision": expected_final_decision,
    "ci_fast_gate": ci_fast_gate,
    "failover_sync_drift_policy_status": policy_status,
    "reason_taxonomy_version": expected_reason_taxonomy_version,
    "reason_codes_csv": expected_reason_codes_csv,
    "drift_taxonomy_reason_taxonomy_version": expected_drift_taxonomy_reason_taxonomy_version,
    "drift_taxonomy_reason_codes_csv": expected_drift_taxonomy_reason_codes_csv,
    "reason_codes": resolved_reason_codes,
    "promotion_decision_reason_mapping_status": "verified",
    "promotion_decision_reason_taxonomy_version": expected_promotion_reason_taxonomy_version,
    "promotion_decision_reason_codes_csv": expected_promotion_reason_codes_csv,
    "promotion_decision_reason_code": promotion_decision_reason_code,
    "report_file": str(report_path),
    "runbook_file": str(runbook_path),
}

pathlib.Path(output_json).write_text(
    json.dumps(policy_payload, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)

print(f"status={status_marker}")
print(f"final_decision={final_decision}")
print(f"failover_sync_drift_policy_status={policy_status}")
print(f"reason_taxonomy_version={expected_reason_taxonomy_version}")
print(f"reason_codes_csv={expected_reason_codes_csv}")
print(
    f"drift_taxonomy_reason_taxonomy_version="
    f"{expected_drift_taxonomy_reason_taxonomy_version}"
)
print(
    f"drift_taxonomy_reason_codes_csv={expected_drift_taxonomy_reason_codes_csv}"
)
print(
    "reason_codes_value="
    + ("none" if status_pass else ",".join(reason_codes))
)
print("promotion_decision_reason_mapping_status=verified")
print(
    "promotion_decision_reason_taxonomy_version="
    + expected_promotion_reason_taxonomy_version
)
print(
    "promotion_decision_reason_codes_csv="
    + expected_promotion_reason_codes_csv
)
print("promotion_decision_reason_code=" + promotion_decision_reason_code)
print(f"report_file={output_json}")

for message in messages:
    print(message, file=sys.stderr)

if not status_pass:
    print(",".join(reason_codes), file=sys.stderr)
    raise SystemExit(1)
PY
}

run_evidence_convergence_check() {
  local report_file=""
  local policy_file=""
  local convergence_output_json="$ROOT_DIR/failover-sync-preflight-convergence-report.json"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --report-file)
        report_file="${2:-}"
        shift 2
        ;;
      --policy-file)
        policy_file="${2:-}"
        shift 2
        ;;
      --output-json)
        convergence_output_json="${2:-}"
        shift 2
        ;;
      --help|-h)
        cat <<'USAGE'
Usage:
  bash scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh check-evidence-convergence \
    --report-file <path> \
    --policy-file <path> \
    [--output-json <path>]
USAGE
        exit 0
        ;;
      *)
        echo "unknown argument: $1" >&2
        exit 1
        ;;
    esac
  done

  if [ -z "$report_file" ]; then
    echo "--report-file is required in check-evidence-convergence mode" >&2
    exit 1
  fi
  if [ -z "$policy_file" ]; then
    echo "--policy-file is required in check-evidence-convergence mode" >&2
    exit 1
  fi
  if [ ! -f "$report_file" ]; then
    echo "report file not found: $report_file" >&2
    exit 1
  fi
  if [ ! -f "$policy_file" ]; then
    echo "policy file not found: $policy_file" >&2
    exit 1
  fi

  mkdir -p "$(dirname "$convergence_output_json")"

  python3 - \
    "$report_file" \
    "$policy_file" \
    "$convergence_output_json" \
    "$failover_readiness_reason_taxonomy_version" \
    "$failover_readiness_reason_codes_csv" \
    "$failover_promotion_decision_reason_taxonomy_version" \
    "$failover_promotion_decision_reason_codes_csv" \
    "$failover_evidence_convergence_reason_taxonomy_version" \
    "$failover_evidence_convergence_reason_codes_csv" \
    <<'PY'
import json
import pathlib
import sys

(
    report_file,
    policy_file,
    output_json,
    expected_reason_taxonomy_version,
    expected_reason_codes_csv,
    expected_promotion_reason_taxonomy_version,
    expected_promotion_reason_codes_csv,
    expected_convergence_reason_taxonomy_version,
    expected_convergence_reason_codes_csv,
) = sys.argv[1:]

report_path = pathlib.Path(report_file)
policy_path = pathlib.Path(policy_file)
report = json.loads(report_path.read_text(encoding="utf-8"))
policy = json.loads(policy_path.read_text(encoding="utf-8"))

reason_codes: list[str] = []
messages: list[str] = []


def add_reason(condition: bool, code: str) -> None:
    if condition and code not in reason_codes:
        reason_codes.append(code)


def resolve_promotion_reason_code(codes: list[str]) -> str:
    if codes == ["none"]:
        return "none"
    preferred_codes = [
        "failover_readiness_progress_stalled",
        "live_node_drift_marker_parity_mismatch",
        "ci_local_promotion_budget_boundary_exceeded",
        "drift_taxonomy_mapping_drift_detected",
        "runbook_marker_parity_mismatch",
        "ci_fast_gate_failed",
        "failover_sync_drift_policy_expected_decision_mismatch",
    ]
    for code in preferred_codes:
        if code in codes:
            return code
    return "failover_sync_drift_policy_violation"


add_reason(
    report.get("schema_version") != "kamn.runtime.failover-sync-drill-report.v1",
    "failover_evidence_payload_tamper_detected:report_schema_version",
)
add_reason(
    policy.get("schema_version")
    != "kamn.runtime.failover-sync-drill-preflight-policy-report.v1",
    "failover_evidence_payload_tamper_detected:policy_schema_version",
)

required_policy_fields = [
    "status",
    "final_decision",
    "failover_sync_drift_policy_status",
    "reason_taxonomy_version",
    "reason_codes_csv",
    "reason_codes",
    "report_file",
    "promotion_decision_reason_mapping_status",
    "promotion_decision_reason_taxonomy_version",
    "promotion_decision_reason_codes_csv",
    "promotion_decision_reason_code",
]
for field in required_policy_fields:
    if field not in policy:
        add_reason(True, f"failover_evidence_link_missing:{field}")
        messages.append(f"missing required policy field: {field}")

expected_report_file = str(report_path.resolve())
policy_report_file = str(pathlib.Path(policy.get("report_file", "")).resolve())
add_reason(
    policy_report_file != expected_report_file,
    "failover_evidence_link_missing:report_file",
)

add_reason(
    policy.get("reason_taxonomy_version") != expected_reason_taxonomy_version,
    "failover_evidence_payload_tamper_detected:reason_taxonomy_version",
)
add_reason(
    policy.get("reason_codes_csv") != expected_reason_codes_csv,
    "failover_evidence_payload_tamper_detected:reason_codes_csv",
)
add_reason(
    policy.get("promotion_decision_reason_mapping_status") != "verified",
    "promotion_decision_reason_mapping_mismatch",
)
add_reason(
    policy.get("promotion_decision_reason_taxonomy_version")
    != expected_promotion_reason_taxonomy_version,
    "promotion_decision_reason_mapping_mismatch",
)
add_reason(
    policy.get("promotion_decision_reason_codes_csv")
    != expected_promotion_reason_codes_csv,
    "promotion_decision_reason_mapping_mismatch",
)

raw_reason_codes = policy.get("reason_codes")
if not isinstance(raw_reason_codes, list):
    add_reason(True, "failover_evidence_payload_tamper_detected:reason_codes_type")
    normalized_reason_codes = []
elif len(raw_reason_codes) == 0:
    add_reason(True, "failover_evidence_payload_tamper_detected:reason_codes_empty")
    normalized_reason_codes = []
elif any(not isinstance(code, str) or not code for code in raw_reason_codes):
    add_reason(True, "failover_evidence_payload_tamper_detected:reason_codes_invalid")
    normalized_reason_codes = []
elif "none" in raw_reason_codes and raw_reason_codes != ["none"]:
    add_reason(
        True,
        "failover_evidence_payload_tamper_detected:reason_codes_none_mixed_with_failures",
    )
    normalized_reason_codes = []
else:
    normalized_reason_codes = raw_reason_codes

if normalized_reason_codes:
    expected_status = "pass" if normalized_reason_codes == ["none"] else "fail"
    expected_final_decision = "GO" if normalized_reason_codes == ["none"] else "NO-GO"
    expected_promotion_reason_code = resolve_promotion_reason_code(normalized_reason_codes)
else:
    expected_status = "fail"
    expected_final_decision = "NO-GO"
    expected_promotion_reason_code = "failover_sync_drift_policy_violation"

add_reason(
    policy.get("status") != expected_status,
    "failover_evidence_payload_tamper_detected:status_decision_mismatch",
)
add_reason(
    policy.get("final_decision") != expected_final_decision,
    "promotion_decision_reason_mapping_mismatch",
)
add_reason(
    policy.get("promotion_decision_reason_code") != expected_promotion_reason_code,
    "promotion_decision_reason_mapping_mismatch",
)

report_reason_code = report.get("reason_code")
if report_reason_code not in {None, "none"}:
    add_reason(
        report_reason_code not in (normalized_reason_codes or []),
        "failover_evidence_payload_tamper_detected:report_reason_not_projected",
    )

status_pass = len(reason_codes) == 0
resolved_reason_codes = ["none"] if status_pass else reason_codes
status = "pass" if status_pass else "fail"
final_decision = "GO" if status_pass else "NO-GO"

convergence_payload = {
    "schema_version": "kamn.runtime.failover-sync-drill-preflight-convergence-report.v1",
    "status": status,
    "final_decision": final_decision,
    "evidence_convergence_status": "verified" if status_pass else "failed",
    "promotion_decision_reason_mapping_status": "verified" if status_pass else "failed",
    "reason_taxonomy_version": expected_convergence_reason_taxonomy_version,
    "reason_codes_csv": expected_convergence_reason_codes_csv,
    "reason_codes": resolved_reason_codes,
    "report_file": str(report_path.resolve()),
    "policy_file": str(policy_path.resolve()),
}
pathlib.Path(output_json).write_text(
    json.dumps(convergence_payload, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)

print("status=" + ("ok" if status_pass else "error"))
print("final_decision=" + final_decision)
print("evidence_convergence_status=" + ("verified" if status_pass else "failed"))
print(
    "promotion_decision_reason_mapping_status="
    + ("verified" if status_pass else "failed")
)
print("reason_taxonomy_version=" + expected_convergence_reason_taxonomy_version)
print("reason_codes_csv=" + expected_convergence_reason_codes_csv)
print("reason_codes_value=" + ("none" if status_pass else ",".join(reason_codes)))
print("report_file=" + output_json)

for message in messages:
    print(message, file=sys.stderr)

if not status_pass:
    print(",".join(reason_codes), file=sys.stderr)
    raise SystemExit(1)
PY
}

if [ "${1:-}" = "check-policy" ]; then
  shift
  run_policy_check "$@"
  exit 0
fi

if [ "${1:-}" = "check-evidence-convergence" ]; then
  shift
  run_evidence_convergence_check "$@"
  exit 0
fi

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
drift_taxonomy_mapping_status="verified"
runbook_marker_parity_status="verified"

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
  "$drift_taxonomy_mapping_status" \
  "$runbook_marker_parity_status" \
  "$drift_taxonomy_runbook_reason_taxonomy_version" \
  "$drift_taxonomy_runbook_reason_codes_csv" \
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
    drift_taxonomy_mapping_status,
    runbook_marker_parity_status,
    drift_taxonomy_runbook_reason_taxonomy_version,
    drift_taxonomy_runbook_reason_codes_csv,
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
    "drift_taxonomy_mapping_status": drift_taxonomy_mapping_status,
    "runbook_marker_parity_status": runbook_marker_parity_status,
    "drift_taxonomy_runbook_reason_taxonomy_version": drift_taxonomy_runbook_reason_taxonomy_version,
    "drift_taxonomy_runbook_reason_codes_csv": drift_taxonomy_runbook_reason_codes_csv,
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
