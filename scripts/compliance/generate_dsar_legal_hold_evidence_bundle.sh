#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/compliance/generate_dsar_legal_hold_evidence_bundle.sh \
    --output-file <path> \
    --request-id <value> \
    --subject-did <value> \
    --request-type ACCESS|EXPORT|ERASURE \
    --legal-hold-active true|false \
    --retention-expired true|false \
    --evidence-complete true|false \
    --approval-recorded true|false \
    --tamper-check PASS|FAIL \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

normalize_bool() {
  local input="$1"
  case "$input" in
    true|false)
      printf '%s\n' "$input"
      ;;
    *)
      fail "boolean fields must be true or false"
      ;;
  esac
}

output_file=""
request_id=""
subject_did=""
request_type=""
legal_hold_active=""
retention_expired=""
evidence_complete=""
approval_recorded=""
tamper_check=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --request-id)
      request_id="${2:-}"
      shift 2
      ;;
    --subject-did)
      subject_did="${2:-}"
      shift 2
      ;;
    --request-type)
      request_type="${2:-}"
      shift 2
      ;;
    --legal-hold-active)
      legal_hold_active="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --retention-expired)
      retention_expired="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --evidence-complete)
      evidence_complete="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --approval-recorded)
      approval_recorded="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --tamper-check)
      tamper_check="${2:-}"
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

if [[ -z "$output_file" || -z "$request_id" || -z "$subject_did" || -z "$request_type" || -z "$legal_hold_active" || -z "$retention_expired" || -z "$evidence_complete" || -z "$approval_recorded" || -z "$tamper_check" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all bundle arguments are required"
fi

case "$request_type" in
  ACCESS|EXPORT|ERASURE) ;;
  *)
    fail "request-type must be ACCESS, EXPORT, or ERASURE"
    ;;
esac

for field in "$tamper_check" "$ci_fast_gate"; do
  if [[ "$field" != "PASS" && "$field" != "FAIL" ]]; then
    fail "tamper-check and ci-fast-gate must be PASS or FAIL"
  fi
done

mkdir -p "$(dirname "$output_file")"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$request_id" "$subject_did" "$request_type" "$legal_hold_active" "$retention_expired" "$evidence_complete" "$approval_recorded" "$tamper_check" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys

(
    output_file,
    generated_at,
    request_id,
    subject_did,
    request_type,
    legal_hold_active_raw,
    retention_expired_raw,
    evidence_complete_raw,
    approval_recorded_raw,
    tamper_check,
    ci_fast_gate,
) = sys.argv[1:]

legal_hold_active = legal_hold_active_raw == "true"
retention_expired = retention_expired_raw == "true"
evidence_complete = evidence_complete_raw == "true"
approval_recorded = approval_recorded_raw == "true"

legal_hold_blocks_erasure = request_type == "ERASURE" and legal_hold_active
retention_allows_erasure = request_type != "ERASURE" or retention_expired

is_go = (
    tamper_check == "PASS"
    and ci_fast_gate == "PASS"
    and evidence_complete
    and approval_recorded
    and not legal_hold_blocks_erasure
    and retention_allows_erasure
)
final_decision = "GO" if is_go else "NO-GO"

reason_codes = []
if tamper_check != "PASS":
    reason_codes.append("tamper_check_failed")
if ci_fast_gate != "PASS":
    reason_codes.append("ci_fast_gate_failed")
if not evidence_complete:
    reason_codes.append("evidence_incomplete")
if not approval_recorded:
    reason_codes.append("approval_missing")
if legal_hold_blocks_erasure:
    reason_codes.append("legal_hold_precedence_block")
if not retention_allows_erasure:
    reason_codes.append("retention_window_not_expired")

payload = {
    "schema_version": "kamn.compliance.dsar-legal-hold.v1",
    "generated_at": generated_at,
    "request_id": request_id,
    "subject_did": subject_did,
    "request_type": request_type,
    "legal_hold_active": legal_hold_active,
    "retention_expired": retention_expired,
    "evidence_complete": evidence_complete,
    "approval_recorded": approval_recorded,
    "tamper_check": tamper_check,
    "ci_fast_gate": ci_fast_gate,
    "policy_checks": {
        "legal_hold_blocks_erasure": legal_hold_blocks_erasure,
        "retention_allows_erasure": retention_allows_erasure,
    },
    "reason_codes": reason_codes,
    "final_decision": final_decision,
}

path = pathlib.Path(output_file)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
print(final_decision)
PY
)"

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'final_decision=%s\n' "$final_decision"

