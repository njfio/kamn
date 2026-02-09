#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/governance/check_governance_simulation_policy.sh \
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
    "simulation_complete",
    "veto_window_open",
    "veto_recorded",
    "timelock_expired",
    "approvals",
    "ci_fast_gate",
    "policy_checks",
    "reason_codes",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

for field in ("simulation_complete", "veto_window_open", "veto_recorded", "timelock_expired"):
    if not isinstance(payload[field], bool):
        fail(f"{field} must be boolean")

if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

approvals = payload["approvals"]
if not isinstance(approvals, dict):
    fail("approvals must be an object")

if "required" not in approvals:
    fail("missing approvals field: required")
if "received" not in approvals:
    fail("missing approvals field: received")

required = approvals["required"]
received = approvals["received"]
if not isinstance(required, int):
    fail("approvals.required must be an integer")
if not isinstance(received, int):
    fail("approvals.received must be an integer")
if required < 1:
    fail("approvals.required must be >= 1")
if received < 0:
    fail("approvals.received must be >= 0")

policy_checks = payload["policy_checks"]
if not isinstance(policy_checks, dict):
    fail("policy_checks must be an object")

if "simulation_hash_valid" not in policy_checks:
    fail("missing policy_checks field: simulation_hash_valid")
if "approval_quorum_met" not in policy_checks:
    fail("missing policy_checks field: approval_quorum_met")

for field in ("simulation_hash_valid", "approval_quorum_met"):
    if not isinstance(policy_checks[field], bool):
        fail(f"policy_checks.{field} must be boolean")

hash_valid = bool(re.match(r"^sha256:[0-9a-f]{64}$", str(payload["simulation_hash"])))
approval_quorum_met = received >= required

if policy_checks["simulation_hash_valid"] != hash_valid:
    fail("policy_checks.simulation_hash_valid does not match derived policy")
if policy_checks["approval_quorum_met"] != approval_quorum_met:
    fail("policy_checks.approval_quorum_met does not match derived policy")

expected_go = (
    payload["simulation_complete"]
    and hash_valid
    and not payload["veto_window_open"]
    and not payload["veto_recorded"]
    and payload["timelock_expired"]
    and approval_quorum_met
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
if not payload["simulation_complete"]:
    failed_checks.append("simulation_missing")
if not hash_valid:
    failed_checks.append("simulation_hash_invalid")
if payload["veto_window_open"]:
    failed_checks.append("veto_window_open")
if payload["veto_recorded"]:
    failed_checks.append("veto_recorded")
if not payload["timelock_expired"]:
    failed_checks.append("timelock_not_expired")
if not approval_quorum_met:
    failed_checks.append("approval_quorum_missing")
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

