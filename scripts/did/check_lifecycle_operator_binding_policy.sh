#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/did/check_lifecycle_operator_binding_policy.sh \
    --bundle-file <path>
USAGE
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
from typing import List


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


bundle_path = pathlib.Path(sys.argv[1])
try:
    payload = json.loads(bundle_path.read_text(encoding="utf-8"))
except json.JSONDecodeError as exc:
    fail(f"bundle file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "generated_at",
    "did",
    "actor_did",
    "required_operator_did",
    "mutation_action",
    "mutation_nonce",
    "mutation_reason_code",
    "audit_export",
    "ci_fast_gate",
    "reason_key",
    "policy_checks",
    "reason_codes",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.did.lifecycle-operator-binding.v1":
    fail("unsupported schema_version for lifecycle operator-binding evidence bundle")

if not isinstance(payload["mutation_nonce"], int):
    fail("mutation_nonce must be an integer")
if payload["mutation_nonce"] <= 0:
    fail("mutation_nonce must be greater than zero")

if payload["mutation_action"] not in {"rotate", "revoke", "recover"}:
    fail("mutation_action must be rotate, revoke, or recover")

if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

known_reason_codes = {
    "did_lifecycle_mutation_allowed",
    "did_lifecycle_mutation_nonce_invalid",
    "did_lifecycle_mutation_nonce_replay",
    "did_lifecycle_mutation_unauthorized_actor",
    "did_lifecycle_mutation_invalid_transition",
}
if not isinstance(payload["mutation_reason_code"], str):
    fail("mutation_reason_code must be a string")

audit_export = payload["audit_export"]
if not isinstance(audit_export, dict):
    fail("audit_export must be an object")
for field in ("export_id", "record_count", "digest"):
    if field not in audit_export:
        fail(f"missing audit_export field: {field}")

if not isinstance(audit_export["record_count"], int):
    fail("audit_export.record_count must be an integer")
if audit_export["record_count"] < 0:
    fail("audit_export.record_count must be non-negative")

hash_pattern = re.compile(r"^sha256:[0-9a-f]{64}$")

operator_binding_satisfied = payload["actor_did"] == payload["required_operator_did"]
mutation_action_supported = payload["mutation_action"] in {"rotate", "revoke", "recover"}
mutation_reason_code_valid = payload["mutation_reason_code"] in known_reason_codes
authorization_granted = (
    operator_binding_satisfied
    and payload["mutation_reason_code"] == "did_lifecycle_mutation_allowed"
)
authorization_evidence_consistent = (
    (
        operator_binding_satisfied
        and payload["mutation_reason_code"] == "did_lifecycle_mutation_allowed"
    )
    or (
        (not operator_binding_satisfied)
        and payload["mutation_reason_code"] == "did_lifecycle_mutation_unauthorized_actor"
    )
)
audit_export_id_present = bool(str(audit_export["export_id"]).strip())
audit_record_count_positive = audit_export["record_count"] > 0
audit_digest_valid = bool(hash_pattern.match(str(audit_export["digest"])))
ci_fast_gate_passed = payload["ci_fast_gate"] == "PASS"

derived_checks = {
    "operator_binding_satisfied": operator_binding_satisfied,
    "mutation_action_supported": mutation_action_supported,
    "mutation_reason_code_valid": mutation_reason_code_valid,
    "authorization_granted": authorization_granted,
    "authorization_evidence_consistent": authorization_evidence_consistent,
    "audit_export_id_present": audit_export_id_present,
    "audit_record_count_positive": audit_record_count_positive,
    "audit_digest_valid": audit_digest_valid,
    "ci_fast_gate_passed": ci_fast_gate_passed,
}

policy_checks = payload["policy_checks"]
if not isinstance(policy_checks, dict):
    fail("policy_checks must be an object")
for field in derived_checks:
    if field not in policy_checks:
        fail(f"missing policy_checks field: {field}")
    if not isinstance(policy_checks[field], bool):
        fail(f"policy_checks.{field} must be boolean")
    if policy_checks[field] != derived_checks[field]:
        fail(f"policy_checks.{field} does not match derived policy")

expected_decision = "GO" if all(derived_checks.values()) else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")
if actual_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}"
    )

reason_key = payload["reason_key"]
if not isinstance(reason_key, str) or not reason_key:
    fail("reason_key must be a non-empty string")
expected_reason_key = f"did_lifecycle_operator_binding_reason_codes:{actual_decision}:v1"
if reason_key != expected_reason_key:
    fail(
        "reason_key mismatch: "
        f"expected {expected_reason_key}, found {reason_key}"
    )

failed_checks: List[str] = []
if not operator_binding_satisfied:
    failed_checks.append("operator_binding_mismatch")
if not mutation_action_supported:
    failed_checks.append("mutation_action_unsupported")
if not mutation_reason_code_valid:
    failed_checks.append("mutation_reason_code_invalid")
if not authorization_granted:
    failed_checks.append("mutation_not_authorized")
if not authorization_evidence_consistent:
    failed_checks.append("authorization_evidence_inconsistent")
if not audit_export_id_present:
    failed_checks.append("audit_export_id_missing")
if not audit_record_count_positive:
    failed_checks.append("audit_record_count_zero")
if not audit_digest_valid:
    failed_checks.append("audit_digest_invalid")
if not ci_fast_gate_passed:
    failed_checks.append("ci_fast_gate_failed")
failed_checks = sorted(failed_checks)

reason_codes = payload["reason_codes"]
if not isinstance(reason_codes, list):
    fail("reason_codes must be an array")
if not all(isinstance(item, str) and item for item in reason_codes):
    fail("reason_codes must contain non-empty strings")
if reason_codes != sorted(reason_codes):
    fail("reason_codes must be sorted and deterministic")
if reason_codes != failed_checks:
    fail(
        "reason_codes mismatch: "
        f"expected reason_codes={failed_checks}, found {reason_codes}"
    )

failed_checks_value = ",".join(failed_checks) if failed_checks else "none"
print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_decision}")
print(f"failed_checks={failed_checks_value}")
PY
)"

printf '%s\n' "$output"
