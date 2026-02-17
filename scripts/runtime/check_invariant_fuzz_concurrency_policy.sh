#!/usr/bin/env bash
set -euo pipefail

REASON_TAXONOMY_VERSION="kamn.runtime.invariant-fuzz-concurrency-policy-reason-taxonomy.v1"
REASON_CODES_CSV="property_lane_failed,fuzz_lane_failed,concurrency_lane_failed,runtime_budget_exceeded,ci_smoke_local_heavy_boundary_status_mismatch,ci_smoke_lane_cost_profile_mismatch,local_heavy_lane_execution_mode_mismatch,missing_required_report_fields,schema_version_mismatch,status_value_invalid,lane_status_value_invalid,property_replay_schema_version_mismatch,property_replay_artifact_key_mismatch,property_replay_test_count_invalid,fuzz_replay_schema_version_mismatch,fuzz_replay_artifact_key_mismatch,fuzz_replay_test_count_invalid,concurrency_replay_schema_version_mismatch,concurrency_replay_artifact_key_mismatch,concurrency_replay_test_count_invalid,elapsed_seconds_invalid,max_seconds_invalid,reason_codes_payload_invalid,status_contract_mismatch,reason_codes_contract_mismatch,reason_taxonomy_version_mismatch,reason_codes_csv_mismatch,reason_codes_value_mismatch,final_decision_mismatch"

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh \
    --report-file <path> \
    [--output-json <path>]
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

report_file=""
output_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --report-file)
      report_file="${2:-}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [[ -z "$report_file" ]]; then
  usage
  fail "--report-file is required"
fi

if [[ ! -f "$report_file" ]]; then
  fail "report file not found: $report_file"
fi

set +e
output="$(
  python3 - "$report_file" "$REASON_TAXONOMY_VERSION" "$REASON_CODES_CSV" "$output_json" <<'PY'
import json
import pathlib
import sys


report_path = pathlib.Path(sys.argv[1])
reason_taxonomy_version = sys.argv[2]
reason_codes_csv = sys.argv[3]
output_json = sys.argv[4]
try:
    payload = json.loads(report_path.read_text(encoding="utf-8"))
except json.JSONDecodeError as exc:
    print(f"report file is not valid JSON: {exc}", file=sys.stderr)
    sys.exit(1)

failure_reason_codes: list[str] = []


def add_reason(code: str) -> None:
    if code not in failure_reason_codes:
        failure_reason_codes.append(code)

required_fields = (
    "schema_version",
    "status",
    "property_lane_status",
    "fuzz_lane_status",
    "concurrency_lane_status",
    "ci_smoke_local_heavy_boundary_status",
    "ci_smoke_lane_cost_profile",
    "local_heavy_lane_execution_mode",
    "property_replay_schema_version",
    "property_replay_artifact_key",
    "property_replay_test_count",
    "fuzz_replay_schema_version",
    "fuzz_replay_artifact_key",
    "fuzz_replay_test_count",
    "concurrency_replay_schema_version",
    "concurrency_replay_artifact_key",
    "concurrency_replay_test_count",
    "elapsed_seconds",
    "max_seconds",
    "reason_codes",
    "reason_taxonomy_version",
    "reason_codes_csv",
    "reason_codes_value",
    "final_decision",
)
missing_fields = [field for field in required_fields if field not in payload]
if missing_fields:
    add_reason("missing_required_report_fields")

schema_version = payload.get("schema_version")
if schema_version != "kamn.runtime.invariant-fuzz-concurrency-contract-report.v1":
    add_reason("schema_version_mismatch")

status = payload.get("status")
if status not in {"pass", "fail"}:
    add_reason("status_value_invalid")

for field in ("property_lane_status", "fuzz_lane_status", "concurrency_lane_status"):
    if payload.get(field) not in {"pass", "fail"}:
        add_reason("lane_status_value_invalid")

if payload.get("ci_smoke_local_heavy_boundary_status") != "verified":
    add_reason("ci_smoke_local_heavy_boundary_status_mismatch")

if payload.get("ci_smoke_lane_cost_profile") != "low":
    add_reason("ci_smoke_lane_cost_profile_mismatch")

if payload.get("local_heavy_lane_execution_mode") != "opt_in":
    add_reason("local_heavy_lane_execution_mode_mismatch")

property_replay_schema_version = payload.get("property_replay_schema_version")
expected_property_replay_schema_version = "kamn.runtime.lifecycle-property-contract-report.v1"
if property_replay_schema_version != expected_property_replay_schema_version:
    add_reason("property_replay_schema_version_mismatch")

property_replay_artifact_key = payload.get("property_replay_artifact_key")
expected_property_replay_artifact_key = "lifecycle_property_replay:v1"
if property_replay_artifact_key != expected_property_replay_artifact_key:
    add_reason("property_replay_artifact_key_mismatch")

property_replay_test_count = payload.get("property_replay_test_count")
if not isinstance(property_replay_test_count, int) or property_replay_test_count < 12:
    add_reason("property_replay_test_count_invalid")

fuzz_replay_schema_version = payload.get("fuzz_replay_schema_version")
expected_fuzz_replay_schema_version = "kamn.runtime.input-mutation-contract-report.v1"
if fuzz_replay_schema_version != expected_fuzz_replay_schema_version:
    add_reason("fuzz_replay_schema_version_mismatch")

fuzz_replay_artifact_key = payload.get("fuzz_replay_artifact_key")
expected_fuzz_replay_artifact_key = "input_mutation_replay:v1"
if fuzz_replay_artifact_key != expected_fuzz_replay_artifact_key:
    add_reason("fuzz_replay_artifact_key_mismatch")

fuzz_replay_test_count = payload.get("fuzz_replay_test_count")
if not isinstance(fuzz_replay_test_count, int) or fuzz_replay_test_count < 10:
    add_reason("fuzz_replay_test_count_invalid")

concurrency_replay_schema_version = payload.get("concurrency_replay_schema_version")
expected_concurrency_replay_schema_version = "kamn.runtime.concurrency-mutation-contract-report.v1"
if concurrency_replay_schema_version != expected_concurrency_replay_schema_version:
    add_reason("concurrency_replay_schema_version_mismatch")

concurrency_replay_artifact_key = payload.get("concurrency_replay_artifact_key")
expected_concurrency_replay_artifact_key = "concurrency_mutation_replay:v1"
if concurrency_replay_artifact_key != expected_concurrency_replay_artifact_key:
    add_reason("concurrency_replay_artifact_key_mismatch")

concurrency_replay_test_count = payload.get("concurrency_replay_test_count")
if not isinstance(concurrency_replay_test_count, int) or concurrency_replay_test_count < 12:
    add_reason("concurrency_replay_test_count_invalid")

elapsed_seconds = payload.get("elapsed_seconds")
max_seconds = payload.get("max_seconds")
if not isinstance(elapsed_seconds, int) or elapsed_seconds < 0:
    add_reason("elapsed_seconds_invalid")
if not isinstance(max_seconds, int) or max_seconds <= 0:
    add_reason("max_seconds_invalid")

reason_codes = payload.get("reason_codes")
observed_reason_codes: list[str] | None = None
observed_reason_codes_value = "<invalid>"
if not isinstance(reason_codes, list) or not reason_codes:
    add_reason("reason_codes_payload_invalid")
elif not all(isinstance(item, str) and item for item in reason_codes):
    add_reason("reason_codes_payload_invalid")
else:
    observed_reason_codes = reason_codes
    observed_reason_codes_value = ",".join(reason_codes)

expected_reason_codes: list[str] = []
if (
    payload.get("property_lane_status") != "pass"
):
    expected_reason_codes.append("property_lane_failed")
if payload.get("fuzz_lane_status") != "pass":
    expected_reason_codes.append("fuzz_lane_failed")
if payload.get("concurrency_lane_status") != "pass":
    expected_reason_codes.append("concurrency_lane_failed")
if isinstance(elapsed_seconds, int) and isinstance(max_seconds, int) and max_seconds > 0 and elapsed_seconds > max_seconds:
    expected_reason_codes.append("runtime_budget_exceeded")

if not expected_reason_codes:
    expected_status = "pass"
    expected_reason_codes = ["none"]
else:
    expected_status = "fail"

expected_reason_codes_value = ",".join(expected_reason_codes)
if observed_reason_codes is not None and observed_reason_codes != expected_reason_codes:
    add_reason("reason_codes_contract_mismatch")

status_contract_mismatch = False
if status in {"pass", "fail"} and status != expected_status:
    add_reason("status_contract_mismatch")
    status_contract_mismatch = True

if payload.get("reason_taxonomy_version") != reason_taxonomy_version:
    add_reason("reason_taxonomy_version_mismatch")

if payload.get("reason_codes_csv") != reason_codes_csv:
    add_reason("reason_codes_csv_mismatch")

if payload.get("reason_codes_value") != observed_reason_codes_value:
    add_reason("reason_codes_value_mismatch")

expected_report_final_decision = "GO" if expected_status == "pass" else "NO-GO"
if not status_contract_mismatch and payload.get("final_decision") != expected_report_final_decision:
    add_reason("final_decision_mismatch")

policy_status = "verified" if not failure_reason_codes else "violation"
canonical_reason_order = reason_codes_csv.split(",")
ordered_failure_reason_codes = [code for code in canonical_reason_order if code in failure_reason_codes]
unknown_failure_reason_codes = [code for code in failure_reason_codes if code not in canonical_reason_order]
ordered_failure_reason_codes.extend(sorted(unknown_failure_reason_codes))
policy_reason_codes = ["none"] if policy_status == "verified" else ordered_failure_reason_codes
policy_reason_codes_value = ",".join(policy_reason_codes)
policy_final_decision = "GO" if policy_status == "verified" else "NO-GO"

if output_json:
    output_path = pathlib.Path(output_json)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_payload = {
        "schema_version": "kamn.runtime.invariant-fuzz-concurrency-policy-report.v1",
        "status": "pass" if policy_status == "verified" else "fail",
        "invariant_policy_status": policy_status,
        "report_file": str(report_path),
        "reason_taxonomy_version": reason_taxonomy_version,
        "reason_codes_csv": reason_codes_csv,
        "reason_codes": policy_reason_codes,
        "reason_codes_value": policy_reason_codes_value,
        "expected_reason_codes_value": expected_reason_codes_value,
        "observed_reason_codes_value": observed_reason_codes_value,
        "ci_smoke_local_heavy_boundary_status": payload.get(
            "ci_smoke_local_heavy_boundary_status"
        ),
        "ci_smoke_lane_cost_profile": payload.get("ci_smoke_lane_cost_profile"),
        "local_heavy_lane_execution_mode": payload.get(
            "local_heavy_lane_execution_mode"
        ),
        "final_decision": policy_final_decision,
    }
    output_path.write_text(json.dumps(output_payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")

print(f"status={'ok' if policy_status == 'verified' else 'fail'}")
print(f"report_file={report_path}")
print(f"final_decision={policy_final_decision}")
print(f"invariant_policy_status={policy_status}")
print(f"invariant_policy_reason_taxonomy_version={reason_taxonomy_version}")
print(f"invariant_policy_reason_codes_csv={reason_codes_csv}")
print(f"invariant_policy_reason_codes_value={policy_reason_codes_value}")
print(f"invariant_policy_expected_reason_codes_value={expected_reason_codes_value}")
print(f"invariant_policy_observed_reason_codes_value={observed_reason_codes_value}")
print(
    "invariant_policy_ci_smoke_local_heavy_boundary_status="
    f"{payload.get('ci_smoke_local_heavy_boundary_status')}"
)
print(
    "invariant_policy_ci_smoke_lane_cost_profile="
    f"{payload.get('ci_smoke_lane_cost_profile')}"
)
print(
    "invariant_policy_local_heavy_lane_execution_mode="
    f"{payload.get('local_heavy_lane_execution_mode')}"
)
print(f"invariant_policy_final_decision={policy_final_decision}")

if policy_status != "verified":
    sys.exit(1)
PY
)"
python_exit=$?
set -e

printf '%s\n' "$output"

if [ "$python_exit" -ne 0 ]; then
  exit "$python_exit"
fi
