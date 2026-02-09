#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/reputation/generate_reputation_dispute_evidence_bundle.sh \
    --output-file <path> \
    --dispute-id <value> \
    --subject-did <value> \
    --reviewer-did <value> \
    --dispute-reason-code QUALITY|DELIVERY|ABUSE|IDENTITY \
    --evidence-uri <value> \
    --evidence-sha256 sha256:<64-hex> \
    --evidence-hash-verified PASS|FAIL \
    --original-trust-score <0-1000> \
    --proposed-trust-score <0-1000> \
    --max-adjustment-points <n> \
    --policy-window-open true|false \
    --approval-recorded true|false \
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
    fail "${name} must be an integer"
  fi
}

output_file=""
dispute_id=""
subject_did=""
reviewer_did=""
dispute_reason_code=""
evidence_uri=""
evidence_sha256=""
evidence_hash_verified=""
original_trust_score=""
proposed_trust_score=""
max_adjustment_points=""
policy_window_open=""
approval_recorded=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --dispute-id)
      dispute_id="${2:-}"
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
    --dispute-reason-code)
      dispute_reason_code="${2:-}"
      shift 2
      ;;
    --evidence-uri)
      evidence_uri="${2:-}"
      shift 2
      ;;
    --evidence-sha256)
      evidence_sha256="${2:-}"
      shift 2
      ;;
    --evidence-hash-verified)
      evidence_hash_verified="${2:-}"
      shift 2
      ;;
    --original-trust-score)
      original_trust_score="${2:-}"
      shift 2
      ;;
    --proposed-trust-score)
      proposed_trust_score="${2:-}"
      shift 2
      ;;
    --max-adjustment-points)
      max_adjustment_points="${2:-}"
      shift 2
      ;;
    --policy-window-open)
      policy_window_open="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --approval-recorded)
      approval_recorded="$(normalize_bool "${2:-}")"
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

if [[ -z "$output_file" || -z "$dispute_id" || -z "$subject_did" || -z "$reviewer_did" || -z "$dispute_reason_code" || -z "$evidence_uri" || -z "$evidence_sha256" || -z "$evidence_hash_verified" || -z "$original_trust_score" || -z "$proposed_trust_score" || -z "$max_adjustment_points" || -z "$policy_window_open" || -z "$approval_recorded" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all bundle arguments are required"
fi

case "$dispute_reason_code" in
  QUALITY|DELIVERY|ABUSE|IDENTITY) ;;
  *)
    fail "dispute-reason-code must be QUALITY, DELIVERY, ABUSE, or IDENTITY"
    ;;
esac

for value in "$original_trust_score" "$proposed_trust_score" "$max_adjustment_points"; do
  require_int "score field" "$value"
done

for value in "$evidence_hash_verified" "$ci_fast_gate"; do
  if [[ "$value" != "PASS" && "$value" != "FAIL" ]]; then
    fail "evidence-hash-verified and ci-fast-gate must be PASS or FAIL"
  fi
done

mkdir -p "$(dirname "$output_file")"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$dispute_id" "$subject_did" "$reviewer_did" "$dispute_reason_code" "$evidence_uri" "$evidence_sha256" "$evidence_hash_verified" "$original_trust_score" "$proposed_trust_score" "$max_adjustment_points" "$policy_window_open" "$approval_recorded" "$ci_fast_gate" <<'PY'
import json
import pathlib
import re
import sys

(
    output_file,
    generated_at,
    dispute_id,
    subject_did,
    reviewer_did,
    dispute_reason_code,
    evidence_uri,
    evidence_sha256,
    evidence_hash_verified,
    original_trust_score_raw,
    proposed_trust_score_raw,
    max_adjustment_points_raw,
    policy_window_open_raw,
    approval_recorded_raw,
    ci_fast_gate,
) = sys.argv[1:]

original_trust_score = int(original_trust_score_raw)
proposed_trust_score = int(proposed_trust_score_raw)
max_adjustment_points = int(max_adjustment_points_raw)
policy_window_open = policy_window_open_raw == "true"
approval_recorded = approval_recorded_raw == "true"
score_delta = abs(proposed_trust_score - original_trust_score)

did_pattern = re.compile(r"^did:[a-z0-9]+:[A-Za-z0-9._:-]+$")
hash_pattern = re.compile(r"^sha256:[0-9a-f]{64}$")
did_fields_valid = bool(did_pattern.match(subject_did) and did_pattern.match(reviewer_did))
evidence_uri_present = len(evidence_uri.strip()) > 0
evidence_hash_valid = bool(hash_pattern.match(evidence_sha256))
evidence_hash_matches = evidence_hash_verified == "PASS"
trust_scores_in_range = (
    0 <= original_trust_score <= 1000 and 0 <= proposed_trust_score <= 1000
)
score_adjustment_within_limit = score_delta <= max_adjustment_points
policy_window_satisfied = policy_window_open
approval_satisfied = approval_recorded
ci_fast_gate_passed = ci_fast_gate == "PASS"

is_go = (
    did_fields_valid
    and evidence_uri_present
    and evidence_hash_valid
    and evidence_hash_matches
    and trust_scores_in_range
    and score_adjustment_within_limit
    and policy_window_satisfied
    and approval_satisfied
    and ci_fast_gate_passed
)
final_decision = "GO" if is_go else "NO-GO"

reason_codes = []
if not did_fields_valid:
    reason_codes.append("did_fields_invalid")
if not evidence_uri_present:
    reason_codes.append("evidence_uri_missing")
if not evidence_hash_valid:
    reason_codes.append("evidence_hash_invalid")
if not evidence_hash_matches:
    reason_codes.append("evidence_hash_verification_failed")
if not trust_scores_in_range:
    reason_codes.append("trust_score_out_of_bounds")
if not score_adjustment_within_limit:
    reason_codes.append("score_adjustment_exceeds_limit")
if not policy_window_satisfied:
    reason_codes.append("policy_window_closed")
if not approval_satisfied:
    reason_codes.append("approval_missing")
if not ci_fast_gate_passed:
    reason_codes.append("ci_fast_gate_failed")

payload = {
    "schema_version": "kamn.reputation.dispute-evidence.v1",
    "generated_at": generated_at,
    "dispute_id": dispute_id,
    "subject_did": subject_did,
    "reviewer_did": reviewer_did,
    "dispute_reason_code": dispute_reason_code,
    "evidence_bundle": {
        "uri": evidence_uri,
        "sha256": evidence_sha256,
        "hash_verified": evidence_hash_verified,
    },
    "score_transition": {
        "original_trust_score": original_trust_score,
        "proposed_trust_score": proposed_trust_score,
        "score_delta": score_delta,
        "max_adjustment_points": max_adjustment_points,
    },
    "policy_window_open": policy_window_open,
    "approval_recorded": approval_recorded,
    "ci_fast_gate": ci_fast_gate,
    "policy_checks": {
        "did_fields_valid": did_fields_valid,
        "evidence_uri_present": evidence_uri_present,
        "evidence_hash_valid": evidence_hash_valid,
        "evidence_hash_matches": evidence_hash_matches,
        "trust_scores_in_range": trust_scores_in_range,
        "score_adjustment_within_limit": score_adjustment_within_limit,
        "policy_window_satisfied": policy_window_satisfied,
        "approval_satisfied": approval_satisfied,
        "ci_fast_gate_passed": ci_fast_gate_passed,
    },
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
printf 'final_decision=%s\n' "$final_decision"
