#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/did/check_federated_did_handshake_policy.sh \
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
    "handshake_id",
    "subject_did",
    "local_network",
    "remote_network",
    "resolver_cache_hit",
    "resolver_version",
    "signature_policy",
    "nonce_monotonic",
    "downgrade_detected",
    "partition_sequence_monotonic",
    "required_quorum",
    "received_quorum",
    "ci_fast_gate",
    "policy_checks",
    "reason_codes",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.did.federated-handshake.v1":
    fail("unsupported schema_version for federated DID handshake bundle")

for field in (
    "resolver_cache_hit",
    "nonce_monotonic",
    "downgrade_detected",
    "partition_sequence_monotonic",
):
    if not isinstance(payload[field], bool):
        fail(f"{field} must be boolean")

for field in ("required_quorum", "received_quorum"):
    if not isinstance(payload[field], int):
        fail(f"{field} must be integer")
    if payload[field] < 0:
        fail(f"{field} must be non-negative")

if payload["required_quorum"] <= 0:
    fail("required_quorum must be greater than zero")

for field in ("signature_policy", "ci_fast_gate"):
    if payload[field] not in {"PASS", "FAIL"}:
        fail(f"{field} must be PASS or FAIL")

policy_checks = payload["policy_checks"]
if not isinstance(policy_checks, dict):
    fail("policy_checks must be an object")

required_policy_fields = (
    "resolver_version_present",
    "signature_policy_passed",
    "quorum_satisfied",
    "replay_guard_passed",
    "downgrade_guard_passed",
)
for field in required_policy_fields:
    if field not in policy_checks:
        fail(f"missing policy_checks field: {field}")
    if not isinstance(policy_checks[field], bool):
        fail(f"policy_checks.{field} must be boolean")

resolver_version_present = bool(str(payload["resolver_version"]).strip())
signature_policy_passed = payload["signature_policy"] == "PASS"
quorum_satisfied = payload["received_quorum"] >= payload["required_quorum"]
replay_guard_passed = payload["nonce_monotonic"] and payload["partition_sequence_monotonic"]
downgrade_guard_passed = not payload["downgrade_detected"]

expected_checks = {
    "resolver_version_present": resolver_version_present,
    "signature_policy_passed": signature_policy_passed,
    "quorum_satisfied": quorum_satisfied,
    "replay_guard_passed": replay_guard_passed,
    "downgrade_guard_passed": downgrade_guard_passed,
}

for key, expected_value in expected_checks.items():
    if policy_checks[key] != expected_value:
        fail(f"policy_checks.{key} does not match derived policy")

expected_go = (
    resolver_version_present
    and signature_policy_passed
    and quorum_satisfied
    and replay_guard_passed
    and downgrade_guard_passed
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
if not resolver_version_present:
    failed_checks.append("resolver_version_missing")
if not signature_policy_passed:
    failed_checks.append("signature_policy_failed")
if not quorum_satisfied:
    failed_checks.append("quorum_shortfall")
if not payload["nonce_monotonic"]:
    failed_checks.append("nonce_replay_detected")
if not payload["partition_sequence_monotonic"]:
    failed_checks.append("partition_sequence_replayed")
if payload["downgrade_detected"]:
    failed_checks.append("downgrade_attack_detected")
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
