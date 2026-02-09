#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/compliance/check_dsar_legal_hold_policy.sh \
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
from typing import List


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
    "request_id",
    "subject_did",
    "request_type",
    "legal_hold_active",
    "retention_expired",
    "evidence_complete",
    "approval_recorded",
    "tamper_check",
    "ci_fast_gate",
    "policy_checks",
    "reason_codes",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

request_type = payload["request_type"]
if request_type not in {"ACCESS", "EXPORT", "ERASURE"}:
    fail("request_type must be ACCESS, EXPORT, or ERASURE")

for field in ("legal_hold_active", "retention_expired", "evidence_complete", "approval_recorded"):
    if not isinstance(payload[field], bool):
        fail(f"{field} must be boolean")

for field in ("tamper_check", "ci_fast_gate"):
    if payload[field] not in {"PASS", "FAIL"}:
        fail(f"{field} must be PASS or FAIL")

policy_checks = payload["policy_checks"]
if not isinstance(policy_checks, dict):
    fail("policy_checks must be an object")

if "legal_hold_blocks_erasure" not in policy_checks:
    fail("missing policy_checks field: legal_hold_blocks_erasure")
if "retention_allows_erasure" not in policy_checks:
    fail("missing policy_checks field: retention_allows_erasure")

for field in ("legal_hold_blocks_erasure", "retention_allows_erasure"):
    if not isinstance(policy_checks[field], bool):
        fail(f"policy_checks.{field} must be boolean")

legal_hold_blocks_erasure = request_type == "ERASURE" and payload["legal_hold_active"]
retention_allows_erasure = request_type != "ERASURE" or payload["retention_expired"]

if policy_checks["legal_hold_blocks_erasure"] != legal_hold_blocks_erasure:
    fail("policy_checks.legal_hold_blocks_erasure does not match derived policy")
if policy_checks["retention_allows_erasure"] != retention_allows_erasure:
    fail("policy_checks.retention_allows_erasure does not match derived policy")

expected_go = (
    payload["tamper_check"] == "PASS"
    and payload["ci_fast_gate"] == "PASS"
    and payload["evidence_complete"]
    and payload["approval_recorded"]
    and not legal_hold_blocks_erasure
    and retention_allows_erasure
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
if payload["tamper_check"] != "PASS":
    failed_checks.append("tamper_check_failed")
if payload["ci_fast_gate"] != "PASS":
    failed_checks.append("ci_fast_gate_failed")
if not payload["evidence_complete"]:
    failed_checks.append("evidence_incomplete")
if not payload["approval_recorded"]:
    failed_checks.append("approval_missing")
if legal_hold_blocks_erasure:
    failed_checks.append("legal_hold_precedence_block")
if not retention_allows_erasure:
    failed_checks.append("retention_window_not_expired")

failed_checks_value = ",".join(failed_checks) if failed_checks else "none"
print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_decision}")
print(f"failed_checks={failed_checks_value}")
PY
)"

printf '%s\n' "$output"

