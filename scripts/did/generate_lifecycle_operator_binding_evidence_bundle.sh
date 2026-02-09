#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/did/generate_lifecycle_operator_binding_evidence_bundle.sh \
    --output-file <path> \
    --did <value> \
    --actor-did <value> \
    --required-operator-did <value> \
    --mutation-action rotate|revoke|recover \
    --mutation-nonce <integer> \
    --mutation-reason-code <value> \
    --audit-export-id <value> \
    --audit-record-count <integer> \
    --audit-digest sha256:<64-hex> \
    --ci-fast-gate PASS|FAIL
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

require_non_negative_int() {
  local name="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "${name} must be a non-negative integer"
  fi
}

output_file=""
did=""
actor_did=""
required_operator_did=""
mutation_action=""
mutation_nonce=""
mutation_reason_code=""
audit_export_id=""
audit_record_count=""
audit_digest=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --did)
      did="${2:-}"
      shift 2
      ;;
    --actor-did)
      actor_did="${2:-}"
      shift 2
      ;;
    --required-operator-did)
      required_operator_did="${2:-}"
      shift 2
      ;;
    --mutation-action)
      mutation_action="${2:-}"
      shift 2
      ;;
    --mutation-nonce)
      mutation_nonce="${2:-}"
      shift 2
      ;;
    --mutation-reason-code)
      mutation_reason_code="${2:-}"
      shift 2
      ;;
    --audit-export-id)
      audit_export_id="${2:-}"
      shift 2
      ;;
    --audit-record-count)
      audit_record_count="${2:-}"
      shift 2
      ;;
    --audit-digest)
      audit_digest="${2:-}"
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

if [[ -z "$output_file" || -z "$did" || -z "$actor_did" || -z "$required_operator_did" || -z "$mutation_action" || -z "$mutation_nonce" || -z "$mutation_reason_code" || -z "$audit_export_id" || -z "$audit_record_count" || -z "$audit_digest" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all lifecycle operator-binding bundle arguments are required"
fi

case "$mutation_action" in
  rotate|revoke|recover) ;;
  *)
    fail "mutation-action must be rotate, revoke, or recover"
    ;;
esac

case "$ci_fast_gate" in
  PASS|FAIL) ;;
  *)
    fail "ci-fast-gate must be PASS or FAIL"
    ;;
esac

require_non_negative_int "mutation-nonce" "$mutation_nonce"
if [[ "$mutation_nonce" -eq 0 ]]; then
  fail "mutation-nonce must be greater than zero"
fi
require_non_negative_int "audit-record-count" "$audit_record_count"

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$did" "$actor_did" "$required_operator_did" "$mutation_action" "$mutation_nonce" "$mutation_reason_code" "$audit_export_id" "$audit_record_count" "$audit_digest" "$ci_fast_gate" <<'PY'
import json
import pathlib
import re
import sys

(
    output_file,
    generated_at,
    did,
    actor_did,
    required_operator_did,
    mutation_action,
    mutation_nonce_raw,
    mutation_reason_code,
    audit_export_id,
    audit_record_count_raw,
    audit_digest,
    ci_fast_gate,
) = sys.argv[1:]

mutation_nonce = int(mutation_nonce_raw)
audit_record_count = int(audit_record_count_raw)

supported_actions = {"rotate", "revoke", "recover"}
known_reason_codes = {
    "did_lifecycle_mutation_allowed",
    "did_lifecycle_mutation_nonce_invalid",
    "did_lifecycle_mutation_nonce_replay",
    "did_lifecycle_mutation_unauthorized_actor",
    "did_lifecycle_mutation_invalid_transition",
}
hash_pattern = re.compile(r"^sha256:[0-9a-f]{64}$")

operator_binding_satisfied = actor_did == required_operator_did
mutation_action_supported = mutation_action in supported_actions
mutation_reason_code_valid = mutation_reason_code in known_reason_codes
authorization_granted = (
    operator_binding_satisfied and mutation_reason_code == "did_lifecycle_mutation_allowed"
)
authorization_evidence_consistent = (
    (operator_binding_satisfied and mutation_reason_code == "did_lifecycle_mutation_allowed")
    or (
        (not operator_binding_satisfied)
        and mutation_reason_code == "did_lifecycle_mutation_unauthorized_actor"
    )
)
audit_export_id_present = bool(audit_export_id.strip())
audit_record_count_positive = audit_record_count > 0
audit_digest_valid = bool(hash_pattern.match(audit_digest))
ci_fast_gate_passed = ci_fast_gate == "PASS"

policy_checks = {
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

reason_codes = []
if not operator_binding_satisfied:
    reason_codes.append("operator_binding_mismatch")
if not mutation_action_supported:
    reason_codes.append("mutation_action_unsupported")
if not mutation_reason_code_valid:
    reason_codes.append("mutation_reason_code_invalid")
if not authorization_granted:
    reason_codes.append("mutation_not_authorized")
if not authorization_evidence_consistent:
    reason_codes.append("authorization_evidence_inconsistent")
if not audit_export_id_present:
    reason_codes.append("audit_export_id_missing")
if not audit_record_count_positive:
    reason_codes.append("audit_record_count_zero")
if not audit_digest_valid:
    reason_codes.append("audit_digest_invalid")
if not ci_fast_gate_passed:
    reason_codes.append("ci_fast_gate_failed")
reason_codes = sorted(reason_codes)

is_go = all(policy_checks.values())
final_decision = "GO" if is_go else "NO-GO"
reason_key = f"did_lifecycle_operator_binding_reason_codes:{final_decision}:v1"

payload = {
    "schema_version": "kamn.did.lifecycle-operator-binding.v1",
    "generated_at": generated_at,
    "did": did,
    "actor_did": actor_did,
    "required_operator_did": required_operator_did,
    "mutation_action": mutation_action,
    "mutation_nonce": mutation_nonce,
    "mutation_reason_code": mutation_reason_code,
    "audit_export": {
        "export_id": audit_export_id,
        "record_count": audit_record_count,
        "digest": audit_digest,
    },
    "ci_fast_gate": ci_fast_gate,
    "reason_key": reason_key,
    "policy_checks": policy_checks,
    "reason_codes": reason_codes,
    "final_decision": final_decision,
}

path = pathlib.Path(output_file)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
print(final_decision)
PY
)"

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'reason_key=%s\n' "did_lifecycle_operator_binding_reason_codes:${final_decision}:v1"
printf 'final_decision=%s\n' "$final_decision"
