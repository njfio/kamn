#!/usr/bin/env bash
set -euo pipefail

report_file=""
expected_final_decision="GO"
ci_fast_gate="PASS"
output_json=""
PACKAGING_REASON_TAXONOMY_VERSION="kamn.deploy.compose-packaging-reason-taxonomy.v1"
PACKAGING_REASON_CODES_CSV="compose_packaging_manifest_drift_detected,compose_packaging_config_drift_detected,compose_packaging_evidence_contract_drift_detected"
POLICY_REASON_TAXONOMY_VERSION="kamn.deploy.compose-topology-contract-policy-reason-taxonomy.v1"

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
      output_json="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$report_file" ]]; then
  echo "--report-file is required" >&2
  exit 1
fi
if [[ "$expected_final_decision" != "GO" && "$expected_final_decision" != "NO-GO" ]]; then
  echo "expected-final-decision must be GO or NO-GO" >&2
  exit 1
fi
if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  echo "ci-fast-gate must be PASS or FAIL" >&2
  exit 1
fi
if [[ ! -f "$report_file" ]]; then
  echo "report file does not exist: $report_file" >&2
  exit 1
fi

python3 - "$report_file" "$expected_final_decision" "$ci_fast_gate" "$output_json" "$PACKAGING_REASON_TAXONOMY_VERSION" "$PACKAGING_REASON_CODES_CSV" "$POLICY_REASON_TAXONOMY_VERSION" <<'PY'
import json
import pathlib
import sys

report_file = pathlib.Path(sys.argv[1])
expected_final_decision = sys.argv[2]
ci_fast_gate = sys.argv[3]
output_json = sys.argv[4]
packaging_reason_taxonomy_version = sys.argv[5]
packaging_reason_codes_csv = sys.argv[6]
policy_reason_taxonomy_version = sys.argv[7]

payload = json.loads(report_file.read_text(encoding="utf-8"))

failed_checks: list[str] = []
if payload.get("schema_version") != "kamn.deploy.compose-topology-contract-lane-summary.v1":
    failed_checks.append("compose_topology_policy_schema_mismatch")
if payload.get("status") != "pass":
    failed_checks.append("compose_topology_policy_status_mismatch")
if payload.get("final_decision") != "GO":
    failed_checks.append("compose_topology_policy_final_decision_mismatch")
if payload.get("ci_fast_gate") != ci_fast_gate:
    failed_checks.append("compose_topology_policy_ci_fast_gate_mismatch")
if payload.get("compose_runtime_mode_full_status") != "verified":
    failed_checks.append("compose_topology_policy_runtime_mode_marker_mismatch")
if payload.get("compose_api_port_status") != "verified":
    failed_checks.append("compose_topology_policy_port_marker_mismatch")
if payload.get("compose_volume_network_status") != "verified":
    failed_checks.append("compose_topology_policy_volume_network_marker_mismatch")
if payload.get("compose_docs_parity_status") != "verified":
    failed_checks.append("compose_topology_policy_docs_marker_mismatch")
if payload.get("compose_manifest_contract_status") != "verified":
    failed_checks.append("compose_topology_policy_compose_manifest_contract_status_mismatch")
if payload.get("compose_config_contract_status") != "verified":
    failed_checks.append("compose_topology_policy_compose_config_contract_status_mismatch")
if payload.get("k8s_manifest_contract_status") != "verified":
    failed_checks.append("compose_topology_policy_k8s_manifest_contract_status_mismatch")
if payload.get("packaging_reason_taxonomy_version") != packaging_reason_taxonomy_version:
    failed_checks.append("compose_topology_policy_packaging_reason_taxonomy_version_mismatch")
if payload.get("packaging_reason_codes_csv") != packaging_reason_codes_csv:
    failed_checks.append("compose_topology_policy_packaging_reason_codes_csv_mismatch")
if payload.get("packaging_contract_evidence_status") != "verified":
    failed_checks.append("compose_topology_policy_packaging_contract_evidence_status_mismatch")
if payload.get("fail_closed_status") != "verified":
    failed_checks.append("compose_topology_policy_fail_closed_status_mismatch")

final_decision = "NO-GO" if failed_checks else "GO"
if final_decision != expected_final_decision:
    failed_checks.append("compose_topology_policy_expected_decision_mismatch")

reason_codes_csv = "none" if not failed_checks else ",".join(failed_checks)
report_payload = {
    "schema_version": "kamn.deploy.compose-topology-contract-policy-report.v1",
    "status": "ok" if not failed_checks else "fail",
    "final_decision": final_decision,
    "expected_final_decision": expected_final_decision,
    "ci_fast_gate": ci_fast_gate,
    "reason_taxonomy_version": policy_reason_taxonomy_version,
    "reason_codes_csv": reason_codes_csv,
    "reason_codes_value": reason_codes_csv,
    "compose_topology_policy_status": "verified" if not failed_checks else "failed",
    "failed_checks": failed_checks,
}

if output_json:
    output_path = pathlib.Path(output_json)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report_payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")

print(f"status={'ok' if not failed_checks else 'fail'}")
print(f"final_decision={final_decision}")
print(f"expected_final_decision={expected_final_decision}")
print(f"ci_fast_gate={ci_fast_gate}")
print(f"reason_taxonomy_version={policy_reason_taxonomy_version}")
print(f"reason_codes_csv={reason_codes_csv}")
print(f"compose_topology_policy_status={'verified' if not failed_checks else 'failed'}")
print(f"failed_checks={','.join(failed_checks)}")

if failed_checks:
    raise SystemExit(",".join(failed_checks))
PY
