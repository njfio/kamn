#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/reputation/check_weighted_decay_property_policy.sh \
    --bundle-file <path>
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

bundle_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundle-file)
      bundle_file="${2:-}"
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

if [[ -z "$bundle_file" ]]; then
  usage
  fail "--bundle-file is required"
fi

if [[ ! -f "$bundle_file" ]]; then
  fail "bundle file not found: $bundle_file"
fi

output="$(
  python3 - "$bundle_file" <<'PY'
import json
import pathlib
import sys
from typing import Any, Dict


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


bundle_path = pathlib.Path(sys.argv[1])
try:
    payload = json.loads(bundle_path.read_text())
except json.JSONDecodeError as exc:
    fail(f"bundle file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "generated_at",
    "lane",
    "evidence_key",
    "reason_key",
    "compact_matrix",
    "adversarial_matrix",
    "anti_gaming_coverage",
    "property_suite_status",
    "runtime_budget_status",
    "ci_fast_gate",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.reputation.weighted-decay.property-evidence.v1":
    fail("unexpected schema_version for weighted decay property evidence bundle")

lane = payload["lane"]
if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")

expected_evidence_key = f"weighted_decay_property_contract:{lane}:v1"
if payload["evidence_key"] != expected_evidence_key:
    fail(
        "evidence_key mismatch: "
        f"expected {expected_evidence_key}, found {payload['evidence_key']}"
    )

property_suite_status = payload["property_suite_status"]
if property_suite_status not in {"pass", "fail"}:
    fail("property_suite_status must be pass or fail")

runtime_budget_status = payload["runtime_budget_status"]
if runtime_budget_status not in {"within", "exceeded"}:
    fail("runtime_budget_status must be within or exceeded")

ci_fast_gate = payload["ci_fast_gate"]
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

decision_reasons = payload["decision_reasons"]
if not isinstance(decision_reasons, list) or not all(
    isinstance(item, str) and item for item in decision_reasons
):
    fail("decision_reasons must be an array of non-empty strings")


def validate_matrix(matrix: Dict[str, Any], label: str) -> Dict[str, Any]:
    if not isinstance(matrix, dict):
        fail(f"{label} matrix must be an object")
    for field in ("status", "case_count", "failed_count", "failed_case_ids", "cases"):
        if field not in matrix:
            fail(f"{label} matrix missing field: {field}")
    if matrix["status"] not in {"pass", "fail"}:
        fail(f"{label} matrix status must be pass or fail")
    if not isinstance(matrix["case_count"], int):
        fail(f"{label} matrix case_count must be an integer")
    if not isinstance(matrix["failed_count"], int):
        fail(f"{label} matrix failed_count must be an integer")
    if not isinstance(matrix["failed_case_ids"], list) or not all(
        isinstance(item, str) and item for item in matrix["failed_case_ids"]
    ):
        fail(f"{label} matrix failed_case_ids must be an array of strings")
    if matrix["failed_case_ids"] != sorted(matrix["failed_case_ids"]):
        fail(f"{label} matrix failed_case_ids must be sorted and deterministic")
    if not isinstance(matrix["cases"], list):
        fail(f"{label} matrix cases must be an array")

    normalized_cases = []
    for index, case in enumerate(matrix["cases"]):
        if not isinstance(case, dict):
            fail(f"{label} matrix case[{index}] must be an object")
        for field in (
            "expected_abuse_penalty_kind",
            "actual_abuse_penalty_kind",
            "passed",
        ):
            if field not in case:
                fail(f"{label} matrix case[{index}] missing field: {field}")
        if not isinstance(case["expected_abuse_penalty_kind"], str) or not case[
            "expected_abuse_penalty_kind"
        ]:
            fail(
                f"{label} matrix case[{index}] expected_abuse_penalty_kind must be a non-empty string"
            )
        if not isinstance(case["actual_abuse_penalty_kind"], str) or not case[
            "actual_abuse_penalty_kind"
        ]:
            fail(
                f"{label} matrix case[{index}] actual_abuse_penalty_kind must be a non-empty string"
            )
        if not isinstance(case["passed"], bool):
            fail(f"{label} matrix case[{index}] passed must be a boolean")
        normalized_cases.append(case)

    return {
        "status": matrix["status"],
        "failed_count": matrix["failed_count"],
        "cases": normalized_cases,
    }


compact = validate_matrix(payload["compact_matrix"], "compact")
adversarial = validate_matrix(payload["adversarial_matrix"], "adversarial")
combined_cases = compact["cases"] + adversarial["cases"]


def observed(kind: str) -> bool:
    return any(
        case["expected_abuse_penalty_kind"] == kind
        and case["actual_abuse_penalty_kind"] == kind
        and case["passed"]
        for case in combined_cases
    )


coverage = payload["anti_gaming_coverage"]
if not isinstance(coverage, dict):
    fail("anti_gaming_coverage must be an object")
for field in (
    "reciprocity_penalty_observed",
    "burst_penalty_observed",
    "churn_penalty_observed",
):
    if field not in coverage:
        fail(f"anti_gaming_coverage missing field: {field}")
    if not isinstance(coverage[field], bool):
        fail(f"anti_gaming_coverage.{field} must be a boolean")

derived_coverage = {
    "reciprocity_penalty_observed": observed("ReciprocityRing"),
    "burst_penalty_observed": observed("BurstSpam"),
    "churn_penalty_observed": observed("ChurnSpike"),
}

for key, value in derived_coverage.items():
    if coverage[key] != value:
        fail(
            f"anti_gaming_coverage mismatch for {key}: expected {value}, found {coverage[key]}"
        )

expected_go = True
if compact["status"] != "pass":
    expected_go = False
if adversarial["status"] != "pass":
    expected_go = False
if compact["failed_count"] != 0:
    expected_go = False
if adversarial["failed_count"] != 0:
    expected_go = False
if not coverage["reciprocity_penalty_observed"]:
    expected_go = False
if not coverage["burst_penalty_observed"]:
    expected_go = False
if not coverage["churn_penalty_observed"]:
    expected_go = False
if property_suite_status != "pass":
    expected_go = False
if runtime_budget_status != "within":
    expected_go = False
if lane == "contract" and ci_fast_gate != "PASS":
    expected_go = False

expected_decision = "GO" if expected_go else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")
if actual_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}"
    )

expected_reason_key = f"weighted_decay_property_contract_reason:{actual_decision}:v1"
if payload["reason_key"] != expected_reason_key:
    fail(
        "reason_key mismatch: "
        f"expected {expected_reason_key}, found {payload['reason_key']}"
    )

print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"schema_version={payload['schema_version']}")
print(f"evidence_key={payload['evidence_key']}")
print(f"reason_key={payload['reason_key']}")
print(f"final_decision={actual_decision}")
PY
)"

printf '%s\n' "$output"
