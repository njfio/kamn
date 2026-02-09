#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/reputation/generate_reputation_recovery_evidence_bundle.sh \
    --output-file <path> \
    --lane contract|deep \
    --recovery-id <value> \
    --subject-did <value> \
    --reviewer-did <value> \
    --pre-penalty-trust-score <0-1000> \
    --post-penalty-trust-score <0-1000> \
    --proposed-recovered-trust-score <0-1000> \
    --max-reversal-points <non-negative-int> \
    --false-positive-confirmed true|false \
    --reviewer-quorum-satisfied true|false \
    --audit-evidence-verified PASS|FAIL \
    --replay-guard-pass true|false \
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

require_int() {
  local name="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "${name} must be a non-negative integer"
  fi
}

output_file=""
lane=""
recovery_id=""
subject_did=""
reviewer_did=""
pre_penalty_trust_score=""
post_penalty_trust_score=""
proposed_recovered_trust_score=""
max_reversal_points=""
false_positive_confirmed=""
reviewer_quorum_satisfied=""
audit_evidence_verified=""
replay_guard_pass=""
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
    --recovery-id)
      recovery_id="${2:-}"
      shift 2
      ;;
    --subject-did)
      subject_did="${2:-}"
      shift 2
      ;;
    --reviewer-did)
      reviewer_did="${2:-}"
      shift 2
      ;;
    --pre-penalty-trust-score)
      pre_penalty_trust_score="${2:-}"
      shift 2
      ;;
    --post-penalty-trust-score)
      post_penalty_trust_score="${2:-}"
      shift 2
      ;;
    --proposed-recovered-trust-score)
      proposed_recovered_trust_score="${2:-}"
      shift 2
      ;;
    --max-reversal-points)
      max_reversal_points="${2:-}"
      shift 2
      ;;
    --false-positive-confirmed)
      false_positive_confirmed="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --reviewer-quorum-satisfied)
      reviewer_quorum_satisfied="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --audit-evidence-verified)
      audit_evidence_verified="${2:-}"
      shift 2
      ;;
    --replay-guard-pass)
      replay_guard_pass="$(normalize_bool "${2:-}")"
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

if [[ -z "$output_file" || -z "$lane" || -z "$recovery_id" || -z "$subject_did" || -z "$reviewer_did" || -z "$pre_penalty_trust_score" || -z "$post_penalty_trust_score" || -z "$proposed_recovered_trust_score" || -z "$max_reversal_points" || -z "$false_positive_confirmed" || -z "$reviewer_quorum_satisfied" || -z "$audit_evidence_verified" || -z "$replay_guard_pass" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all recovery evidence bundle arguments are required"
fi

case "$lane" in
  contract|deep) ;;
  *)
    fail "lane must be contract or deep"
    ;;
esac

case "$audit_evidence_verified" in
  PASS|FAIL) ;;
  *)
    fail "audit-evidence-verified must be PASS or FAIL"
    ;;
esac

case "$ci_fast_gate" in
  PASS|FAIL) ;;
  *)
    fail "ci-fast-gate must be PASS or FAIL"
    ;;
esac

for value_name in \
  pre_penalty_trust_score \
  post_penalty_trust_score \
  proposed_recovered_trust_score \
  max_reversal_points; do
  require_int "$value_name" "${!value_name}"
done

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$lane" "$recovery_id" "$subject_did" "$reviewer_did" "$pre_penalty_trust_score" "$post_penalty_trust_score" "$proposed_recovered_trust_score" "$max_reversal_points" "$false_positive_confirmed" "$reviewer_quorum_satisfied" "$audit_evidence_verified" "$replay_guard_pass" "$ci_fast_gate" <<'PY'
import json
import pathlib
import re
import sys
from typing import Dict, List


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    lane,
    recovery_id,
    subject_did,
    reviewer_did,
    pre_penalty_trust_score_raw,
    post_penalty_trust_score_raw,
    proposed_recovered_trust_score_raw,
    max_reversal_points_raw,
    false_positive_confirmed_raw,
    reviewer_quorum_satisfied_raw,
    audit_evidence_verified,
    replay_guard_pass_raw,
    ci_fast_gate,
) = sys.argv[1:]

if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")
if audit_evidence_verified not in {"PASS", "FAIL"}:
    fail("audit_evidence_verified must be PASS or FAIL")
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

pre_penalty_trust_score = int(pre_penalty_trust_score_raw)
post_penalty_trust_score = int(post_penalty_trust_score_raw)
proposed_recovered_trust_score = int(proposed_recovered_trust_score_raw)
max_reversal_points = int(max_reversal_points_raw)
false_positive_confirmed = false_positive_confirmed_raw == "true"
reviewer_quorum_satisfied = reviewer_quorum_satisfied_raw == "true"
replay_guard_passed = replay_guard_pass_raw == "true"

did_pattern = re.compile(r"^did:[a-z0-9]+:[A-Za-z0-9._:-]+$")
did_fields_valid = bool(
    did_pattern.match(subject_did) and did_pattern.match(reviewer_did)
)

reversal_points = proposed_recovered_trust_score - post_penalty_trust_score
reversal_within_limit = 0 <= reversal_points <= max_reversal_points
restored_score_within_bounds = (
    0 <= pre_penalty_trust_score <= 1000
    and 0 <= post_penalty_trust_score <= 1000
    and 0 <= proposed_recovered_trust_score <= 1000
    and post_penalty_trust_score <= proposed_recovered_trust_score <= pre_penalty_trust_score
)

policy_checks: Dict[str, bool] = {
    "did_fields_valid": did_fields_valid,
    "false_positive_confirmed": false_positive_confirmed,
    "reviewer_quorum_satisfied": reviewer_quorum_satisfied,
    "audit_evidence_verified": audit_evidence_verified == "PASS",
    "replay_guard_passed": replay_guard_passed,
    "reversal_within_limit": reversal_within_limit,
    "restored_score_within_bounds": restored_score_within_bounds,
    "ci_fast_gate_passed": ci_fast_gate == "PASS",
}

is_go = all(policy_checks.values())
final_decision = "GO" if is_go else "NO-GO"
recovery_action = "REVERSE_PENALTY" if is_go else "HOLD_PENALTY"

reason_codes: List[str] = []
if not policy_checks["did_fields_valid"]:
    reason_codes.append("did_fields_invalid")
if not policy_checks["false_positive_confirmed"]:
    reason_codes.append("false_positive_not_confirmed")
if not policy_checks["reviewer_quorum_satisfied"]:
    reason_codes.append("reviewer_quorum_missing")
if not policy_checks["audit_evidence_verified"]:
    reason_codes.append("audit_evidence_verification_failed")
if not policy_checks["replay_guard_passed"]:
    reason_codes.append("replay_guard_nonce_reused")
if not policy_checks["reversal_within_limit"]:
    reason_codes.append("reversal_exceeds_limit")
if not policy_checks["restored_score_within_bounds"]:
    reason_codes.append("restored_score_out_of_bounds")
if not policy_checks["ci_fast_gate_passed"]:
    reason_codes.append("ci_fast_gate_failed")
reason_codes = sorted(reason_codes)

reason_key = f"reputation_recovery_reason_codes:{final_decision}:v1"
evidence_key = f"reputation_recovery_reversal_contract:{lane}:v1"

payload = {
    "schema_version": "kamn.reputation.recovery-reversal-evidence.v1",
    "generated_at": generated_at,
    "lane": lane,
    "evidence_key": evidence_key,
    "reason_key": reason_key,
    "recovery_context": {
        "recovery_id": recovery_id,
        "subject_did": subject_did,
        "reviewer_did": reviewer_did,
    },
    "score_transition": {
        "pre_penalty_trust_score": pre_penalty_trust_score,
        "post_penalty_trust_score": post_penalty_trust_score,
        "proposed_recovered_trust_score": proposed_recovered_trust_score,
        "reversal_points": reversal_points,
        "max_reversal_points": max_reversal_points,
    },
    "recovery_controls": {
        "false_positive_confirmed": false_positive_confirmed,
        "reviewer_quorum_satisfied": reviewer_quorum_satisfied,
        "audit_evidence_verified": audit_evidence_verified,
        "replay_guard_passed": replay_guard_passed,
        "ci_fast_gate": ci_fast_gate,
    },
    "policy_checks": policy_checks,
    "reason_codes": reason_codes,
    "recovery_action": recovery_action,
    "final_decision": final_decision,
}

path = pathlib.Path(output_file)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
print(final_decision)
PY
)"

recovery_action="HOLD_PENALTY"
if [ "$final_decision" = "GO" ]; then
  recovery_action="REVERSE_PENALTY"
fi

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'schema_version=kamn.reputation.recovery-reversal-evidence.v1\n'
printf 'evidence_key=reputation_recovery_reversal_contract:%s:v1\n' "$lane"
printf 'reason_key=reputation_recovery_reason_codes:%s:v1\n' "$final_decision"
printf 'recovery_action=%s\n' "$recovery_action"
printf 'final_decision=%s\n' "$final_decision"
