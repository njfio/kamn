#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/compliance/check_soc2_control_evidence_policy.sh \
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
import re
import sys
from typing import Dict, List


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
    "control_id",
    "audit_period",
    "collector_did",
    "evidence_uri",
    "evidence_sha256",
    "checks",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

audit_period = payload["audit_period"]
if not isinstance(audit_period, dict):
    fail("bundle field 'audit_period' must be an object")

for field in ("start", "end"):
    if field not in audit_period:
        fail(f"missing audit_period field: {field}")
    if not isinstance(audit_period[field], str):
        fail(f"audit_period.{field} must be a string")

checks = payload["checks"]
if not isinstance(checks, dict):
    fail("bundle field 'checks' must be an object")

for field in ("tamper", "completeness", "ci_fast_gate"):
    if field not in checks:
        fail(f"missing checks field: {field}")
    if checks[field] not in {"PASS", "FAIL"}:
        fail(f"checks.{field} must be PASS or FAIL")

for field in ("period_valid", "hash_valid"):
    if field not in checks:
        fail(f"missing checks field: {field}")
    if not isinstance(checks[field], bool):
        fail(f"checks.{field} must be boolean")

audit_start = audit_period["start"]
audit_end = audit_period["end"]
date_pattern = re.compile(r"^\d{4}-\d{2}-\d{2}$")
date_values_valid = bool(date_pattern.match(audit_start)) and bool(
    date_pattern.match(audit_end)
)
period_valid = date_values_valid and audit_start <= audit_end and checks["period_valid"]

hash_valid = bool(re.match(r"^sha256:[0-9a-f]{64}$", str(payload["evidence_sha256"])))
if checks["hash_valid"] != hash_valid:
    fail("checks.hash_valid must match evidence_sha256 format validation")

expected_go = (
    checks["tamper"] == "PASS"
    and checks["completeness"] == "PASS"
    and checks["ci_fast_gate"] == "PASS"
    and period_valid
    and hash_valid
)
expected_decision = "GO" if expected_go else "NO-GO"
actual_decision = payload["final_decision"]

if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")

if actual_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}"
    )

failed_checks: List[str] = []
if checks["tamper"] != "PASS":
    failed_checks.append("tamper")
if checks["completeness"] != "PASS":
    failed_checks.append("completeness")
if checks["ci_fast_gate"] != "PASS":
    failed_checks.append("ci_fast_gate")
if not period_valid:
    failed_checks.append("period_valid")
if not hash_valid:
    failed_checks.append("hash_valid")

failed_checks_value = ",".join(failed_checks) if failed_checks else "none"
print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_decision}")
print(f"failed_checks={failed_checks_value}")
PY
)"

printf '%s\n' "$output"

