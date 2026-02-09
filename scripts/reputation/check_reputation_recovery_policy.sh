#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/reputation/check_reputation_recovery_policy.sh \
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
    payload = json.loads(bundle_path.read_text(encoding="utf-8"))
except json.JSONDecodeError as exc:
    fail(f"bundle file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "generated_at",
    "lane",
    "evidence_key",
    "reason_key",
    "recovery_context",
    "score_transition",
    "recovery_controls",
    "policy_checks",
    "reason_codes",
    "recovery_action",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.reputation.recovery-reversal-evidence.v1":
    fail("unexpected schema_version for reputation recovery evidence bundle")

lane = payload["lane"]
if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")

expected_evidence_key = f"reputation_recovery_reversal_contract:{lane}:v1"
if payload["evidence_key"] != expected_evidence_key:
    fail(
        "evidence_key mismatch: "
        f"expected {expected_evidence_key}, found {payload['evidence_key']}"
    )

recovery_context = payload["recovery_context"]
if not isinstance(recovery_context, dict):
    fail("recovery_context must be an object")
for field in ("recovery_id", "subject_did", "reviewer_did"):
    if field not in recovery_context:
        fail(f"recovery_context missing field: {field}")
    if not isinstance(recovery_context[field], str) or not recovery_context[field]:
        fail(f"recovery_context.{field} must be a non-empty string")

score_transition = payload["score_transition"]
if not isinstance(score_transition, dict):
    fail("score_transition must be an object")
for field in (
    "pre_penalty_trust_score",
    "post_penalty_trust_score",
    "proposed_recovered_trust_score",
    "reversal_points",
    "max_reversal_points",
):
    if field not in score_transition:
        fail(f"score_transition missing field: {field}")
    if not isinstance(score_transition[field], int):
        fail(f"score_transition.{field} must be an integer")

recovery_controls = payload["recovery_controls"]
if not isinstance(recovery_controls, dict):
    fail("recovery_controls must be an object")
for field in (
    "false_positive_confirmed",
    "reviewer_quorum_satisfied",
    "audit_evidence_verified",
    "replay_guard_passed",
    "ci_fast_gate",
):
    if field not in recovery_controls:
        fail(f"recovery_controls missing field: {field}")
if not isinstance(recovery_controls["false_positive_confirmed"], bool):
    fail("recovery_controls.false_positive_confirmed must be boolean")
if not isinstance(recovery_controls["reviewer_quorum_satisfied"], bool):
    fail("recovery_controls.reviewer_quorum_satisfied must be boolean")
if recovery_controls["audit_evidence_verified"] not in {"PASS", "FAIL"}:
    fail("recovery_controls.audit_evidence_verified must be PASS or FAIL")
if not isinstance(recovery_controls["replay_guard_passed"], bool):
    fail("recovery_controls.replay_guard_passed must be boolean")
if recovery_controls["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("recovery_controls.ci_fast_gate must be PASS or FAIL")

policy_checks = payload["policy_checks"]
if not isinstance(policy_checks, dict):
    fail("policy_checks must be an object")
required_checks = (
    "did_fields_valid",
    "false_positive_confirmed",
    "reviewer_quorum_satisfied",
    "audit_evidence_verified",
    "replay_guard_passed",
    "reversal_within_limit",
    "restored_score_within_bounds",
    "ci_fast_gate_passed",
)
for field in required_checks:
    if field not in policy_checks:
        fail(f"policy_checks missing field: {field}")
    if not isinstance(policy_checks[field], bool):
        fail(f"policy_checks.{field} must be boolean")

did_pattern = re.compile(r"^did:[a-z0-9]+:[A-Za-z0-9._:-]+$")
pre_penalty = score_transition["pre_penalty_trust_score"]
post_penalty = score_transition["post_penalty_trust_score"]
proposed_recovered = score_transition["proposed_recovered_trust_score"]
reversal_points = score_transition["reversal_points"]
max_reversal_points = score_transition["max_reversal_points"]
derived_reversal_points = proposed_recovered - post_penalty
if reversal_points != derived_reversal_points:
    fail(
        "score_transition.reversal_points does not match derived score delta: "
        f"expected {derived_reversal_points}, found {reversal_points}"
    )

derived_checks: Dict[str, bool] = {
    "did_fields_valid": bool(
        did_pattern.match(recovery_context["subject_did"])
        and did_pattern.match(recovery_context["reviewer_did"])
    ),
    "false_positive_confirmed": recovery_controls["false_positive_confirmed"],
    "reviewer_quorum_satisfied": recovery_controls["reviewer_quorum_satisfied"],
    "audit_evidence_verified": recovery_controls["audit_evidence_verified"] == "PASS",
    "replay_guard_passed": recovery_controls["replay_guard_passed"],
    "reversal_within_limit": 0 <= reversal_points <= max_reversal_points,
    "restored_score_within_bounds": (
        0 <= pre_penalty <= 1000
        and 0 <= post_penalty <= 1000
        and 0 <= proposed_recovered <= 1000
        and post_penalty <= proposed_recovered <= pre_penalty
    ),
    "ci_fast_gate_passed": recovery_controls["ci_fast_gate"] == "PASS",
}

for key, value in derived_checks.items():
    if policy_checks[key] != value:
        fail(f"policy_checks.{key} does not match derived policy")

expected_decision = "GO" if all(derived_checks.values()) else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")
if actual_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}"
    )

expected_action = "REVERSE_PENALTY" if actual_decision == "GO" else "HOLD_PENALTY"
recovery_action = payload["recovery_action"]
if recovery_action not in {"REVERSE_PENALTY", "HOLD_PENALTY"}:
    fail("recovery_action must be REVERSE_PENALTY or HOLD_PENALTY")
if recovery_action != expected_action:
    fail(
        "recovery_action mismatch: "
        f"expected {expected_action}, found {recovery_action}"
    )

reason_key = payload["reason_key"]
if not isinstance(reason_key, str) or not reason_key:
    fail("reason_key must be a non-empty string")
expected_reason_key = f"reputation_recovery_reason_codes:{actual_decision}:v1"
if reason_key != expected_reason_key:
    fail(
        "reason_key mismatch: "
        f"expected {expected_reason_key}, found {reason_key}"
    )

failed_checks: List[str] = []
if not derived_checks["did_fields_valid"]:
    failed_checks.append("did_fields_invalid")
if not derived_checks["false_positive_confirmed"]:
    failed_checks.append("false_positive_not_confirmed")
if not derived_checks["reviewer_quorum_satisfied"]:
    failed_checks.append("reviewer_quorum_missing")
if not derived_checks["audit_evidence_verified"]:
    failed_checks.append("audit_evidence_verification_failed")
if not derived_checks["replay_guard_passed"]:
    failed_checks.append("replay_guard_nonce_reused")
if not derived_checks["reversal_within_limit"]:
    failed_checks.append("reversal_exceeds_limit")
if not derived_checks["restored_score_within_bounds"]:
    failed_checks.append("restored_score_out_of_bounds")
if not derived_checks["ci_fast_gate_passed"]:
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
print(f"schema_version={payload['schema_version']}")
print(f"evidence_key={payload['evidence_key']}")
print(f"reason_key={payload['reason_key']}")
print(f"final_decision={actual_decision}")
print(f"recovery_action={recovery_action}")
print(f"failed_checks={failed_checks_value}")
PY
)"

printf '%s\n' "$output"
