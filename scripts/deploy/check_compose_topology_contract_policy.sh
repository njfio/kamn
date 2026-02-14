#!/usr/bin/env bash
set -euo pipefail

report_file=""
expected_final_decision="GO"
ci_fast_gate="PASS"
output_json=""

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

python3 - "$report_file" "$expected_final_decision" "$ci_fast_gate" "$output_json" <<'PY'
import json
import pathlib
import sys

report_file = pathlib.Path(sys.argv[1])
expected_final_decision = sys.argv[2]
ci_fast_gate = sys.argv[3]
output_json = sys.argv[4]

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
if payload.get("fail_closed_status") != "verified":
    failed_checks.append("compose_topology_policy_fail_closed_status_mismatch")

final_decision = "NO-GO" if failed_checks else "GO"
if final_decision != expected_final_decision:
    failed_checks.append("compose_topology_policy_expected_decision_mismatch")

report_payload = {
    "schema_version": "kamn.deploy.compose-topology-contract-policy-report.v1",
    "status": "ok" if not failed_checks else "fail",
    "final_decision": final_decision,
    "expected_final_decision": expected_final_decision,
    "ci_fast_gate": ci_fast_gate,
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
print(f"compose_topology_policy_status={'verified' if not failed_checks else 'failed'}")
print(f"failed_checks={','.join(failed_checks)}")

if failed_checks:
    raise SystemExit(",".join(failed_checks))
PY
