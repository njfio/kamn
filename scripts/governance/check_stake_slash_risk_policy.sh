#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/governance/check_stake_slash_risk_policy.sh \
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
    payload = json.loads(bundle_path.read_text())
except json.JSONDecodeError as exc:
    fail(f"bundle file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "generated_at",
    "proposal_id",
    "simulation_hash",
    "risk_metrics_bps",
    "risk_thresholds_bps",
    "evidence_complete",
    "ci_fast_gate",
    "policy_checks",
    "reason_codes",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if not isinstance(payload["evidence_complete"], bool):
    fail("evidence_complete must be boolean")

if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

metrics = payload["risk_metrics_bps"]
thresholds = payload["risk_thresholds_bps"]
if not isinstance(metrics, dict):
    fail("risk_metrics_bps must be an object")
if not isinstance(thresholds, dict):
    fail("risk_thresholds_bps must be an object")

metric_fields = (
    "stake_at_risk",
    "slash_probability",
    "validator_churn",
    "quorum_safety_margin",
)
threshold_fields = (
    "max_stake_at_risk",
    "max_slash_probability",
    "max_validator_churn",
    "min_quorum_safety_margin",
)

for field in metric_fields:
    if field not in metrics:
        fail(f"missing risk_metrics_bps field: {field}")
    if not isinstance(metrics[field], int):
        fail(f"risk_metrics_bps.{field} must be an integer")

for field in threshold_fields:
    if field not in thresholds:
        fail(f"missing risk_thresholds_bps field: {field}")
    if not isinstance(thresholds[field], int):
        fail(f"risk_thresholds_bps.{field} must be an integer")

policy_checks = payload["policy_checks"]
if not isinstance(policy_checks, dict):
    fail("policy_checks must be an object")

for field in (
    "simulation_hash_valid",
    "stake_risk_within_limit",
    "slash_probability_within_limit",
    "validator_churn_within_limit",
    "quorum_margin_within_limit",
):
    if field not in policy_checks:
        fail(f"missing policy_checks field: {field}")
    if not isinstance(policy_checks[field], bool):
        fail(f"policy_checks.{field} must be boolean")

hash_valid = bool(re.match(r"^sha256:[0-9a-f]{64}$", str(payload["simulation_hash"])))
stake_risk_within_limit = metrics["stake_at_risk"] <= thresholds["max_stake_at_risk"]
slash_probability_within_limit = (
    metrics["slash_probability"] <= thresholds["max_slash_probability"]
)
validator_churn_within_limit = metrics["validator_churn"] <= thresholds["max_validator_churn"]
quorum_margin_within_limit = (
    metrics["quorum_safety_margin"] >= thresholds["min_quorum_safety_margin"]
)

if policy_checks["simulation_hash_valid"] != hash_valid:
    fail("policy_checks.simulation_hash_valid does not match derived policy")
if policy_checks["stake_risk_within_limit"] != stake_risk_within_limit:
    fail("policy_checks.stake_risk_within_limit does not match derived policy")
if policy_checks["slash_probability_within_limit"] != slash_probability_within_limit:
    fail("policy_checks.slash_probability_within_limit does not match derived policy")
if policy_checks["validator_churn_within_limit"] != validator_churn_within_limit:
    fail("policy_checks.validator_churn_within_limit does not match derived policy")
if policy_checks["quorum_margin_within_limit"] != quorum_margin_within_limit:
    fail("policy_checks.quorum_margin_within_limit does not match derived policy")

expected_go = (
    hash_valid
    and stake_risk_within_limit
    and slash_probability_within_limit
    and validator_churn_within_limit
    and quorum_margin_within_limit
    and payload["evidence_complete"]
    and payload["ci_fast_gate"] == "PASS"
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
if not hash_valid:
    failed_checks.append("simulation_hash_invalid")
if not stake_risk_within_limit:
    failed_checks.append("stake_at_risk_threshold_breach")
if not slash_probability_within_limit:
    failed_checks.append("slash_probability_threshold_breach")
if not validator_churn_within_limit:
    failed_checks.append("validator_churn_threshold_breach")
if not quorum_margin_within_limit:
    failed_checks.append("quorum_safety_margin_breach")
if not payload["evidence_complete"]:
    failed_checks.append("evidence_incomplete")
if payload["ci_fast_gate"] != "PASS":
    failed_checks.append("ci_fast_gate_failed")

failed_checks_value = ",".join(failed_checks) if failed_checks else "none"
print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_decision}")
print(f"failed_checks={failed_checks_value}")
PY
)"

printf '%s\n' "$output"

