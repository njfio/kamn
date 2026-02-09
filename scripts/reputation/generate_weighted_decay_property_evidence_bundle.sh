#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/reputation/generate_weighted_decay_property_evidence_bundle.sh \
    --output-file <path> \
    --lane contract|deep \
    --compact-report-file <path> \
    --adversarial-report-file <path> \
    --property-suite-status pass|fail \
    --runtime-budget-status within|exceeded \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

output_file=""
lane=""
compact_report_file=""
adversarial_report_file=""
property_suite_status=""
runtime_budget_status=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --lane)
      lane="${2:-}"
      shift 2
      ;;
    --compact-report-file)
      compact_report_file="${2:-}"
      shift 2
      ;;
    --adversarial-report-file)
      adversarial_report_file="${2:-}"
      shift 2
      ;;
    --property-suite-status)
      property_suite_status="${2:-}"
      shift 2
      ;;
    --runtime-budget-status)
      runtime_budget_status="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
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

if [[ -z "$output_file" || -z "$lane" || -z "$compact_report_file" || -z "$adversarial_report_file" || -z "$property_suite_status" || -z "$runtime_budget_status" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all weighted decay property evidence bundle arguments are required"
fi

if [[ ! -f "$compact_report_file" ]]; then
  fail "compact report file not found: $compact_report_file"
fi

if [[ ! -f "$adversarial_report_file" ]]; then
  fail "adversarial report file not found: $adversarial_report_file"
fi

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$lane" "$compact_report_file" "$adversarial_report_file" "$property_suite_status" "$runtime_budget_status" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys
from typing import Any, Dict, List


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    lane,
    compact_report_file,
    adversarial_report_file,
    property_suite_status,
    runtime_budget_status,
    ci_fast_gate,
) = sys.argv[1:]

if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")
if property_suite_status not in {"pass", "fail"}:
    fail("property-suite-status must be pass or fail")
if runtime_budget_status not in {"within", "exceeded"}:
    fail("runtime-budget-status must be within or exceeded")
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci-fast-gate must be PASS or FAIL")


def parse_report(path: str, label: str) -> Dict[str, Any]:
    report = json.loads(pathlib.Path(path).read_text())
    if report.get("schema_version") != "kamn.reputation.weighted-decay.matrix.v1":
        fail(f"{label} report schema_version mismatch")
    status = report.get("status")
    if status not in {"pass", "fail"}:
        fail(f"{label} report status must be pass or fail")
    case_count = report.get("case_count")
    failed_count = report.get("failed_count")
    failed_case_ids = report.get("failed_case_ids")
    cases = report.get("cases")

    if not isinstance(case_count, int):
        fail(f"{label} report case_count must be an integer")
    if not isinstance(failed_count, int):
        fail(f"{label} report failed_count must be an integer")
    if not isinstance(failed_case_ids, list) or not all(
        isinstance(item, str) and item for item in failed_case_ids
    ):
        fail(f"{label} report failed_case_ids must be an array of non-empty strings")
    if failed_case_ids != sorted(failed_case_ids):
        fail(f"{label} report failed_case_ids must be sorted and deterministic")
    if not isinstance(cases, list):
        fail(f"{label} report cases must be an array")

    normalized_cases: List[Dict[str, Any]] = []
    for index, case in enumerate(cases):
        if not isinstance(case, dict):
            fail(f"{label} report case[{index}] must be an object")
        expected_kind = case.get("expected_abuse_penalty_kind")
        actual_kind = case.get("actual_abuse_penalty_kind")
        passed = case.get("passed")
        if not isinstance(expected_kind, str) or not expected_kind:
            fail(f"{label} report case[{index}] expected_abuse_penalty_kind must be a string")
        if not isinstance(actual_kind, str) or not actual_kind:
            fail(f"{label} report case[{index}] actual_abuse_penalty_kind must be a string")
        if not isinstance(passed, bool):
            fail(f"{label} report case[{index}] passed must be a boolean")
        normalized_cases.append(
            {
                "expected_abuse_penalty_kind": expected_kind,
                "actual_abuse_penalty_kind": actual_kind,
                "passed": passed,
            }
        )

    return {
        "status": status,
        "case_count": case_count,
        "failed_count": failed_count,
        "failed_case_ids": failed_case_ids,
        "cases": normalized_cases,
    }


compact = parse_report(compact_report_file, "compact")
adversarial = parse_report(adversarial_report_file, "adversarial")
combined_cases = compact["cases"] + adversarial["cases"]


def penalty_observed(kind: str) -> bool:
    return any(
        case["expected_abuse_penalty_kind"] == kind
        and case["actual_abuse_penalty_kind"] == kind
        and case["passed"]
        for case in combined_cases
    )


reciprocity_penalty_observed = penalty_observed("ReciprocityRing")
burst_penalty_observed = penalty_observed("BurstSpam")
churn_penalty_observed = penalty_observed("ChurnSpike")

decision_reasons: List[str] = []
if compact["status"] != "pass":
    decision_reasons.append("compact_matrix_not_pass")
if adversarial["status"] != "pass":
    decision_reasons.append("adversarial_matrix_not_pass")
if compact["failed_count"] != 0:
    decision_reasons.append("compact_matrix_failures_present")
if adversarial["failed_count"] != 0:
    decision_reasons.append("adversarial_matrix_failures_present")
if not reciprocity_penalty_observed:
    decision_reasons.append("reciprocity_penalty_not_observed")
if not burst_penalty_observed:
    decision_reasons.append("burst_penalty_not_observed")
if not churn_penalty_observed:
    decision_reasons.append("churn_penalty_not_observed")
if property_suite_status != "pass":
    decision_reasons.append("property_suite_failed")
if runtime_budget_status != "within":
    decision_reasons.append("runtime_budget_exceeded")
if lane == "contract" and ci_fast_gate != "PASS":
    decision_reasons.append("ci_fast_gate_failed")

final_decision = "GO" if not decision_reasons else "NO-GO"
if not decision_reasons:
    decision_reasons.append("all weighted decay anti-gaming property checks passed")

evidence_key = f"weighted_decay_property_contract:{lane}:v1"
reason_key = f"weighted_decay_property_contract_reason:{final_decision}:v1"

payload = {
    "schema_version": "kamn.reputation.weighted-decay.property-evidence.v1",
    "generated_at": generated_at,
    "lane": lane,
    "evidence_key": evidence_key,
    "reason_key": reason_key,
    "compact_matrix": compact,
    "adversarial_matrix": adversarial,
    "anti_gaming_coverage": {
        "reciprocity_penalty_observed": reciprocity_penalty_observed,
        "burst_penalty_observed": burst_penalty_observed,
        "churn_penalty_observed": churn_penalty_observed,
    },
    "property_suite_status": property_suite_status,
    "runtime_budget_status": runtime_budget_status,
    "ci_fast_gate": ci_fast_gate,
    "decision_reasons": decision_reasons,
    "final_decision": final_decision,
}

path = pathlib.Path(output_file)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
print(final_decision)
PY
)"

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'schema_version=kamn.reputation.weighted-decay.property-evidence.v1\n'
printf 'evidence_key=weighted_decay_property_contract:%s:v1\n' "$lane"
printf 'reason_key=weighted_decay_property_contract_reason:%s:v1\n' "$final_decision"
printf 'final_decision=%s\n' "$final_decision"
