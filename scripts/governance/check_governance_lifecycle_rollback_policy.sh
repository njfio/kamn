#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/governance/check_governance_lifecycle_rollback_policy.sh \
    --report-file <path>
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

report_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --report-file)
      report_file="${2:-}"
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

output="$(
  python3 - "$report_file" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


report_path = pathlib.Path(sys.argv[1])
try:
    payload = json.loads(report_path.read_text())
except json.JSONDecodeError as exc:
    fail(f"report file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "generated_at",
    "max_runtime_seconds",
    "runtime_seconds",
    "checks",
    "commands",
    "decision_reasons",
    "final_decision",
    "reason_key",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing report field: {field}")

if payload["schema_version"] != "kamn.governance.lifecycle-rollback-report.v1":
    fail("unexpected governance lifecycle/rollback report schema_version")

if not isinstance(payload["max_runtime_seconds"], int) or payload["max_runtime_seconds"] < 0:
    fail("max_runtime_seconds must be an integer >= 0")
if not isinstance(payload["runtime_seconds"], int) or payload["runtime_seconds"] < 0:
    fail("runtime_seconds must be an integer >= 0")

checks = payload["checks"]
if not isinstance(checks, dict):
    fail("checks must be an object")
for field in (
    "lane_failed",
    "lifecycle_contract_present",
    "rollback_contract_present",
    "docs_contract_present",
    "runtime_budget_ok",
):
    if field not in checks:
        fail(f"missing checks field: {field}")
    if not isinstance(checks[field], bool):
        fail(f"checks.{field} must be a boolean")

commands = payload["commands"]
if not isinstance(commands, list) or any(not isinstance(item, str) for item in commands):
    fail("commands must be an array of strings")

actual_reasons = payload["decision_reasons"]
if not isinstance(actual_reasons, list) or any(not isinstance(item, str) for item in actual_reasons):
    fail("decision_reasons must be an array of strings")

runtime_budget_ok_expected = payload["runtime_seconds"] <= payload["max_runtime_seconds"]
if checks["runtime_budget_ok"] != runtime_budget_ok_expected:
    fail(
        "checks.runtime_budget_ok mismatch: "
        f"expected {runtime_budget_ok_expected}, found {checks['runtime_budget_ok']}"
    )

expected_reasons = []
if checks["lane_failed"]:
    expected_reasons.append("governance_lifecycle_lane_failed")
if not checks["lifecycle_contract_present"]:
    expected_reasons.append("lifecycle_contract_missing")
if not checks["rollback_contract_present"]:
    expected_reasons.append("rollback_contract_missing")
if not checks["docs_contract_present"]:
    expected_reasons.append("docs_contract_missing")
if not runtime_budget_ok_expected:
    expected_reasons.append("runtime_budget_exceeded")

if actual_reasons != expected_reasons:
    fail(
        "decision_reasons mismatch: "
        f"expected {expected_reasons}, found {actual_reasons}"
    )

expected_decision = "GO" if not expected_reasons else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")
if actual_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}"
    )

expected_reason_key = f"governance_lifecycle_rollback_reason_codes:{expected_decision}:v1"
actual_reason_key = payload["reason_key"]
if actual_reason_key != expected_reason_key:
    fail(
        "reason_key mismatch: "
        f"expected {expected_reason_key}, found {actual_reason_key}"
    )

print("status=ok")
print(f"report_file={report_path}")
print(f"final_decision={actual_decision}")
print(f"reason_key={actual_reason_key}")
print(f"runtime_seconds={payload['runtime_seconds']}")
print(f"max_runtime_seconds={payload['max_runtime_seconds']}")
PY
)"

printf '%s\n' "$output"
