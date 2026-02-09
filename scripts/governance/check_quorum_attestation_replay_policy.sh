#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/governance/check_quorum_attestation_replay_policy.sh \
    --report-file <path>
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

report_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --report-file)
      report_file="${2:-}"
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

if [[ -z "$report_file" ]]; then
  usage
  fail "--report-file is required"
fi

if [[ ! -f "$report_file" ]]; then
  fail "report file not found: $report_file"
fi

output="$(
  python3 - "$report_file" <<'PY'
import json
import pathlib
import re
import sys


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


report_path = pathlib.Path(sys.argv[1])
try:
    payload = json.loads(report_path.read_text())
except json.JSONDecodeError as exc:
    fail(f"report file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "generated_at",
    "max_runtime_seconds",
    "runtime_seconds",
    "attestation_bundle",
    "checks",
    "commands",
    "decision_reasons",
    "final_decision",
    "reason_key",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing report field: {field}")

if payload["schema_version"] != "kamn.governance.quorum-attestation-replay-report.v1":
    fail("unexpected governance quorum attestation report schema_version")

max_runtime_seconds = payload["max_runtime_seconds"]
runtime_seconds = payload["runtime_seconds"]
if not isinstance(max_runtime_seconds, int) or max_runtime_seconds < 0:
    fail("max_runtime_seconds must be an integer >= 0")
if not isinstance(runtime_seconds, int) or runtime_seconds < 0:
    fail("runtime_seconds must be an integer >= 0")

bundle = payload["attestation_bundle"]
if not isinstance(bundle, dict):
    fail("attestation_bundle must be an object")

for field in (
    "proposal_id",
    "approval_artifact_id",
    "payload_hash",
    "approver_dids",
    "required_signatures",
    "received_signatures",
    "replay_detected",
    "signature_metadata",
):
    if field not in bundle:
        fail(f"missing attestation_bundle field: {field}")

proposal_id = bundle["proposal_id"]
approval_artifact_id = bundle["approval_artifact_id"]
payload_hash = bundle["payload_hash"]
approver_dids = bundle["approver_dids"]
required_signatures = bundle["required_signatures"]
received_signatures = bundle["received_signatures"]
replay_detected = bundle["replay_detected"]
signature_metadata = bundle["signature_metadata"]

if not isinstance(proposal_id, str):
    fail("attestation_bundle.proposal_id must be a string")
if not isinstance(approval_artifact_id, str):
    fail("attestation_bundle.approval_artifact_id must be a string")
if not isinstance(payload_hash, str):
    fail("attestation_bundle.payload_hash must be a string")
if not isinstance(approver_dids, list) or any(not isinstance(item, str) for item in approver_dids):
    fail("attestation_bundle.approver_dids must be an array of strings")
if not isinstance(required_signatures, int) or required_signatures < 0:
    fail("attestation_bundle.required_signatures must be an integer >= 0")
if not isinstance(received_signatures, int) or received_signatures < 0:
    fail("attestation_bundle.received_signatures must be an integer >= 0")
if not isinstance(replay_detected, bool):
    fail("attestation_bundle.replay_detected must be boolean")
if not isinstance(signature_metadata, dict):
    fail("attestation_bundle.signature_metadata must be an object")

for field in ("algorithm", "key_id", "signed_at_unix"):
    if field not in signature_metadata:
        fail(f"missing signature_metadata field: {field}")

algorithm = signature_metadata["algorithm"]
key_id = signature_metadata["key_id"]
signed_at_unix = signature_metadata["signed_at_unix"]
if not isinstance(algorithm, str):
    fail("signature_metadata.algorithm must be a string")
if not isinstance(key_id, str):
    fail("signature_metadata.key_id must be a string")
if not isinstance(signed_at_unix, int):
    fail("signature_metadata.signed_at_unix must be an integer")

checks = payload["checks"]
if not isinstance(checks, dict):
    fail("checks must be an object")
for field in (
    "lane_failed",
    "required_keys_present",
    "signature_metadata_valid",
    "approval_quorum_met",
    "replay_guard_passed",
    "docs_contract_present",
    "runtime_budget_ok",
):
    if field not in checks:
        fail(f"missing checks field: {field}")
    if not isinstance(checks[field], bool):
        fail(f"checks.{field} must be boolean")

commands = payload["commands"]
if not isinstance(commands, list) or any(not isinstance(item, str) for item in commands):
    fail("commands must be an array of strings")

actual_reasons = payload["decision_reasons"]
if not isinstance(actual_reasons, list) or any(not isinstance(item, str) for item in actual_reasons):
    fail("decision_reasons must be an array of strings")

required_keys_present = (
    bool(proposal_id)
    and bool(approval_artifact_id)
    and bool(payload_hash)
    and bool(re.match(r"^sha256:[0-9a-f]{64}$", payload_hash))
    and len(approver_dids) > 0
    and all(did.startswith("kamn:did:agent:") and len(did) > len("kamn:did:agent:") for did in approver_dids)
)

signature_metadata_valid = (
    algorithm in {"ed25519", "secp256k1"}
    and bool(key_id)
    and signed_at_unix > 0
)

approval_quorum_met = required_signatures >= 1 and received_signatures >= required_signatures
replay_guard_passed = not replay_detected
runtime_budget_ok = runtime_seconds <= max_runtime_seconds

if checks["required_keys_present"] != required_keys_present:
    fail(
        "checks.required_keys_present mismatch: "
        f"expected {required_keys_present}, found {checks['required_keys_present']}"
    )
if checks["signature_metadata_valid"] != signature_metadata_valid:
    fail(
        "checks.signature_metadata_valid mismatch: "
        f"expected {signature_metadata_valid}, found {checks['signature_metadata_valid']}"
    )
if checks["approval_quorum_met"] != approval_quorum_met:
    fail(
        "checks.approval_quorum_met mismatch: "
        f"expected {approval_quorum_met}, found {checks['approval_quorum_met']}"
    )
if checks["replay_guard_passed"] != replay_guard_passed:
    fail(
        "checks.replay_guard_passed mismatch: "
        f"expected {replay_guard_passed}, found {checks['replay_guard_passed']}"
    )
if checks["runtime_budget_ok"] != runtime_budget_ok:
    fail(
        "checks.runtime_budget_ok mismatch: "
        f"expected {runtime_budget_ok}, found {checks['runtime_budget_ok']}"
    )

expected_reasons = []
if checks["lane_failed"]:
    expected_reasons.append("governance_quorum_lane_failed")
if not required_keys_present:
    expected_reasons.append("quorum_attestation_required_keys_missing")
if not signature_metadata_valid:
    expected_reasons.append("quorum_attestation_signature_metadata_invalid")
if not approval_quorum_met:
    expected_reasons.append("quorum_attestation_approval_quorum_missing")
if not replay_guard_passed:
    expected_reasons.append("quorum_attestation_replay_detected")
if not checks["docs_contract_present"]:
    expected_reasons.append("quorum_attestation_docs_contract_missing")
if not runtime_budget_ok:
    expected_reasons.append("runtime_budget_exceeded")

if actual_reasons != expected_reasons:
    fail(
        "decision_reasons mismatch: "
        f"expected {expected_reasons}, found {actual_reasons}"
    )

expected_decision = "GO" if not expected_reasons else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")
if actual_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}"
    )

expected_reason_key = f"governance_quorum_attestation_reason_codes:{expected_decision}:v1"
actual_reason_key = payload["reason_key"]
if actual_reason_key != expected_reason_key:
    fail(
        "reason_key mismatch: "
        f"expected {expected_reason_key}, found {actual_reason_key}"
    )

print("status=ok")
print(f"report_file={report_path}")
print(f"final_decision={actual_decision}")
print(f"reason_key={actual_reason_key}")
print(f"runtime_seconds={runtime_seconds}")
print(f"max_runtime_seconds={max_runtime_seconds}")
PY
)"

printf '%s\n' "$output"
