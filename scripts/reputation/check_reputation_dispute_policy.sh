#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/reputation/check_reputation_dispute_policy.sh \
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
    "dispute_id",
    "subject_did",
    "reviewer_did",
    "dispute_reason_code",
    "evidence_bundle",
    "score_transition",
    "policy_window_open",
    "approval_recorded",
    "ci_fast_gate",
    "policy_checks",
    "reason_codes",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["dispute_reason_code"] not in {"QUALITY", "DELIVERY", "ABUSE", "IDENTITY"}:
    fail("dispute_reason_code must be QUALITY, DELIVERY, ABUSE, or IDENTITY")

for field in ("policy_window_open", "approval_recorded"):
    if not isinstance(payload[field], bool):
        fail(f"{field} must be boolean")

if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

evidence_bundle = payload["evidence_bundle"]
if not isinstance(evidence_bundle, dict):
    fail("evidence_bundle must be an object")
for field in ("uri", "sha256", "hash_verified"):
    if field not in evidence_bundle:
        fail(f"missing evidence_bundle field: {field}")
if evidence_bundle["hash_verified"] not in {"PASS", "FAIL"}:
    fail("evidence_bundle.hash_verified must be PASS or FAIL")
if not isinstance(evidence_bundle["uri"], str):
    fail("evidence_bundle.uri must be a string")
if not isinstance(evidence_bundle["sha256"], str):
    fail("evidence_bundle.sha256 must be a string")

score_transition = payload["score_transition"]
if not isinstance(score_transition, dict):
    fail("score_transition must be an object")
for field in (
    "original_trust_score",
    "proposed_trust_score",
    "score_delta",
    "max_adjustment_points",
):
    if field not in score_transition:
        fail(f"missing score_transition field: {field}")
    if not isinstance(score_transition[field], int):
        fail(f"score_transition.{field} must be an integer")

policy_checks = payload["policy_checks"]
if not isinstance(policy_checks, dict):
    fail("policy_checks must be an object")
for field in (
    "did_fields_valid",
    "evidence_uri_present",
    "evidence_hash_valid",
    "evidence_hash_matches",
    "trust_scores_in_range",
    "score_adjustment_within_limit",
    "policy_window_satisfied",
    "approval_satisfied",
    "ci_fast_gate_passed",
):
    if field not in policy_checks:
        fail(f"missing policy_checks field: {field}")
    if not isinstance(policy_checks[field], bool):
        fail(f"policy_checks.{field} must be boolean")

did_pattern = re.compile(r"^did:[a-z0-9]+:[A-Za-z0-9._:-]+$")
hash_pattern = re.compile(r"^sha256:[0-9a-f]{64}$")

subject_did = str(payload["subject_did"])
reviewer_did = str(payload["reviewer_did"])
did_fields_valid = bool(did_pattern.match(subject_did) and did_pattern.match(reviewer_did))
evidence_uri_present = len(evidence_bundle["uri"].strip()) > 0
evidence_hash_valid = bool(hash_pattern.match(evidence_bundle["sha256"]))
evidence_hash_matches = evidence_bundle["hash_verified"] == "PASS"

original_score = score_transition["original_trust_score"]
proposed_score = score_transition["proposed_trust_score"]
score_delta = score_transition["score_delta"]
max_adjustment_points = score_transition["max_adjustment_points"]
trust_scores_in_range = 0 <= original_score <= 1000 and 0 <= proposed_score <= 1000
derived_score_delta = abs(proposed_score - original_score)
if score_delta != derived_score_delta:
    fail(
        "score_transition.score_delta does not match derived score delta: "
        f"expected {derived_score_delta}, found {score_delta}"
    )
score_adjustment_within_limit = derived_score_delta <= max_adjustment_points
policy_window_satisfied = payload["policy_window_open"]
approval_satisfied = payload["approval_recorded"]
ci_fast_gate_passed = payload["ci_fast_gate"] == "PASS"

derived_checks = {
    "did_fields_valid": did_fields_valid,
    "evidence_uri_present": evidence_uri_present,
    "evidence_hash_valid": evidence_hash_valid,
    "evidence_hash_matches": evidence_hash_matches,
    "trust_scores_in_range": trust_scores_in_range,
    "score_adjustment_within_limit": score_adjustment_within_limit,
    "policy_window_satisfied": policy_window_satisfied,
    "approval_satisfied": approval_satisfied,
    "ci_fast_gate_passed": ci_fast_gate_passed,
}

for key, value in derived_checks.items():
    if policy_checks[key] != value:
        fail(f"policy_checks.{key} does not match derived policy")

expected_go = all(derived_checks.values())
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
if not did_fields_valid:
    failed_checks.append("did_fields_invalid")
if not evidence_uri_present:
    failed_checks.append("evidence_uri_missing")
if not evidence_hash_valid:
    failed_checks.append("evidence_hash_invalid")
if not evidence_hash_matches:
    failed_checks.append("evidence_hash_verification_failed")
if not trust_scores_in_range:
    failed_checks.append("trust_score_out_of_bounds")
if not score_adjustment_within_limit:
    failed_checks.append("score_adjustment_exceeds_limit")
if not policy_window_satisfied:
    failed_checks.append("policy_window_closed")
if not approval_satisfied:
    failed_checks.append("approval_missing")
if not ci_fast_gate_passed:
    failed_checks.append("ci_fast_gate_failed")

failed_checks_value = ",".join(failed_checks) if failed_checks else "none"
print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_decision}")
print(f"failed_checks={failed_checks_value}")
PY
)"

printf '%s\n' "$output"
