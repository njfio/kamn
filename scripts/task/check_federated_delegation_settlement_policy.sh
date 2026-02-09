#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/task/check_federated_delegation_settlement_policy.sh \
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
import sys


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
    "delegation_id",
    "task_id",
    "delegator_did",
    "delegatee_did",
    "source_network",
    "destination_network",
    "settlement_reference_id",
    "expected_settlement_reference_id",
    "settlement_receipt_finality",
    "nonce_monotonic",
    "replay_detected",
    "partition_sequence_monotonic",
    "required_attestors",
    "received_attestors",
    "ci_fast_gate",
    "policy_checks",
    "reason_codes",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.task.federated-delegation-settlement.v1":
    fail("unsupported schema_version for federated delegation settlement bundle")

if payload["settlement_receipt_finality"] not in {"FINAL", "PENDING", "FAILED"}:
    fail("settlement_receipt_finality must be FINAL|PENDING|FAILED")

for field in ("ci_fast_gate",):
    if payload[field] not in {"PASS", "FAIL"}:
        fail(f"{field} must be PASS or FAIL")

for field in ("nonce_monotonic", "replay_detected", "partition_sequence_monotonic"):
    if not isinstance(payload[field], bool):
        fail(f"{field} must be boolean")

for field in ("required_attestors", "received_attestors"):
    if not isinstance(payload[field], int):
        fail(f"{field} must be integer")
    if payload[field] < 0:
        fail(f"{field} must be non-negative")

if payload["required_attestors"] <= 0:
    fail("required_attestors must be greater than zero")

policy_checks = payload["policy_checks"]
if not isinstance(policy_checks, dict):
    fail("policy_checks must be an object")

required_policy_fields = (
    "delegation_context_present",
    "settlement_reference_present",
    "settlement_reference_match",
    "receipt_finality_final",
    "replay_guard_passed",
    "quorum_satisfied",
    "cross_network_delegation",
)
for field in required_policy_fields:
    if field not in policy_checks:
        fail(f"missing policy_checks field: {field}")
    if not isinstance(policy_checks[field], bool):
        fail(f"policy_checks.{field} must be boolean")

delegation_context_present = all(
    bool(str(payload[field]).strip())
    for field in ("delegation_id", "task_id", "delegator_did", "delegatee_did")
)
settlement_reference_present = bool(str(payload["settlement_reference_id"]).strip()) and bool(
    str(payload["expected_settlement_reference_id"]).strip()
)
settlement_reference_match = (
    payload["settlement_reference_id"] == payload["expected_settlement_reference_id"]
)
receipt_finality_final = payload["settlement_receipt_finality"] == "FINAL"
replay_guard_passed = (
    payload["nonce_monotonic"]
    and (not payload["replay_detected"])
    and payload["partition_sequence_monotonic"]
)
quorum_satisfied = payload["received_attestors"] >= payload["required_attestors"]
cross_network_delegation = (
    bool(str(payload["source_network"]).strip())
    and bool(str(payload["destination_network"]).strip())
    and payload["source_network"] != payload["destination_network"]
)

expected_checks = {
    "delegation_context_present": delegation_context_present,
    "settlement_reference_present": settlement_reference_present,
    "settlement_reference_match": settlement_reference_match,
    "receipt_finality_final": receipt_finality_final,
    "replay_guard_passed": replay_guard_passed,
    "quorum_satisfied": quorum_satisfied,
    "cross_network_delegation": cross_network_delegation,
}
for key, expected_value in expected_checks.items():
    if policy_checks[key] != expected_value:
        fail(f"policy_checks.{key} does not match derived policy")

expected_go = (
    delegation_context_present
    and settlement_reference_present
    and settlement_reference_match
    and receipt_finality_final
    and replay_guard_passed
    and quorum_satisfied
    and cross_network_delegation
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

failed_checks: list[str] = []
if not delegation_context_present:
    failed_checks.append("delegation_context_missing")
if not settlement_reference_present:
    failed_checks.append("settlement_reference_missing")
if settlement_reference_present and not settlement_reference_match:
    failed_checks.append("settlement_reference_drift")
if not receipt_finality_final:
    failed_checks.append("settlement_receipt_not_final")
if not payload["nonce_monotonic"]:
    failed_checks.append("nonce_not_monotonic")
if payload["replay_detected"]:
    failed_checks.append("replay_detected")
if not payload["partition_sequence_monotonic"]:
    failed_checks.append("partition_sequence_replayed")
if not quorum_satisfied:
    failed_checks.append("attestor_quorum_shortfall")
if not cross_network_delegation:
    failed_checks.append("non_federated_network_pair")
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
