#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/dashboard/check_backend_session_auth_freshness_policy.sh \
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
from typing import List


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))

required_fields = (
    "schema_version",
    "evidence_key",
    "status",
    "final_decision",
    "reason_key",
    "elapsed_seconds",
    "max_seconds",
    "skip_commands",
    "dashboard_package_exit_code",
    "command_count",
    "commands",
    "frontend_contract_passed",
    "session_guard_passed",
    "freshness_guard_passed",
    "docs_contract_passed",
    "reason_codes",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing field: {field}")

if payload["schema_version"] != "kamn.dashboard.backend-session-auth-freshness-report.v1":
    fail("unexpected schema_version for dashboard backend session/auth freshness report")
if payload["evidence_key"] != "dashboard_backend_session_auth_freshness:v1":
    fail("unexpected evidence_key for dashboard backend session/auth freshness report")

status = payload["status"]
if status not in {"pass", "fail"}:
    fail("status must be pass or fail")

final_decision = payload["final_decision"]
if final_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")

expected_reason_key = f"dashboard_backend_session_auth_freshness_reason_codes:{final_decision}:v1"
if payload["reason_key"] != expected_reason_key:
    fail(
        "reason_key mismatch: "
        f"expected {expected_reason_key}, found {payload['reason_key']}"
    )

if not isinstance(payload["elapsed_seconds"], int):
    fail("elapsed_seconds must be an integer")
if not isinstance(payload["max_seconds"], int):
    fail("max_seconds must be an integer")
if payload["max_seconds"] < 0:
    fail("max_seconds must be non-negative")

if not isinstance(payload["skip_commands"], bool):
    fail("skip_commands must be boolean")
if not isinstance(payload["dashboard_package_exit_code"], int):
    fail("dashboard_package_exit_code must be an integer")

commands = payload["commands"]
if not isinstance(commands, list):
    fail("commands must be an array")
if not all(isinstance(item, str) and item for item in commands):
    fail("commands must contain non-empty strings")

command_count = payload["command_count"]
if not isinstance(command_count, int):
    fail("command_count must be an integer")
if command_count != len(commands):
    fail("command_count must match commands length")

for field in (
    "frontend_contract_passed",
    "session_guard_passed",
    "freshness_guard_passed",
    "docs_contract_passed",
):
    if not isinstance(payload[field], bool):
        fail(f"{field} must be boolean")

reason_codes = payload["reason_codes"]
if not isinstance(reason_codes, list):
    fail("reason_codes must be an array")
if not all(isinstance(item, str) and item for item in reason_codes):
    fail("reason_codes must contain non-empty strings")
if reason_codes != sorted(reason_codes):
    fail("reason_codes must be sorted and deterministic")

expected_reasons: List[str] = []
if not payload["frontend_contract_passed"]:
    expected_reasons.append("backend_lane_failed")
if not payload["session_guard_passed"]:
    expected_reasons.append("session_guard_missing")
if not payload["freshness_guard_passed"]:
    expected_reasons.append("freshness_guard_missing")
if not payload["docs_contract_passed"]:
    expected_reasons.append("docs_contract_missing")
if payload["elapsed_seconds"] > payload["max_seconds"]:
    expected_reasons.append("runtime_budget_exceeded")
expected_reasons = sorted(expected_reasons)

expected_status = "pass" if not expected_reasons else "fail"
expected_decision = "GO" if not expected_reasons else "NO-GO"

if status != expected_status:
    fail(f"status mismatch: expected {expected_status}, found {status}")
if final_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {final_decision}"
    )
if reason_codes != expected_reasons:
    fail(
        "reason_codes mismatch: "
        f"expected reason_codes={expected_reasons}, found {reason_codes}"
    )

failed_checks = ",".join(expected_reasons) if expected_reasons else "none"
print("status=ok")
print(f"report_file={path}")
print(f"reason_key={payload['reason_key']}")
print(f"final_decision={final_decision}")
print(f"failed_checks={failed_checks}")
PY
)"

printf '%s\n' "$output"
