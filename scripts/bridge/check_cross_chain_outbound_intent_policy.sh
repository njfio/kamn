#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/bridge/check_cross_chain_outbound_intent_policy.sh \
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
    "chain",
    "request_id",
    "destination_channel",
    "approvals",
    "retry",
    "ci_fast_gate",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.bridge.cross-chain-outbound-intent.v1":
    fail("unexpected schema_version for cross-chain outbound intent evidence bundle")

chain = payload["chain"]
if chain not in {"ethereum", "near"}:
    fail("chain must be ethereum or near")

request_id = payload["request_id"]
if not isinstance(request_id, str) or not request_id.strip():
    fail("request_id must be a non-empty string")

destination_channel = payload["destination_channel"]
if not isinstance(destination_channel, str):
    fail("destination_channel must be a string")
if not destination_channel.startswith(f"{chain}:"):
    fail("destination channel must match selected chain prefix")

approvals = payload["approvals"]
if not isinstance(approvals, dict):
    fail("approvals must be an object")
for key in ("required", "received", "approval_quorum_hash"):
    if key not in approvals:
        fail(f"approvals missing field: {key}")
if not isinstance(approvals["required"], int) or approvals["required"] < 0:
    fail("approvals.required must be a non-negative integer")
if not isinstance(approvals["received"], int) or approvals["received"] < 0:
    fail("approvals.received must be a non-negative integer")
if not isinstance(approvals["approval_quorum_hash"], str):
    fail("approvals.approval_quorum_hash must be a string")

retry = payload["retry"]
if not isinstance(retry, dict):
    fail("retry must be an object")
for key in (
    "idempotency_key",
    "attempt_number",
    "payload_hash",
    "previous_payload_hash",
    "duplicate_request",
):
    if key not in retry:
        fail(f"retry missing field: {key}")
if not isinstance(retry["idempotency_key"], str):
    fail("retry.idempotency_key must be a string")
if not isinstance(retry["attempt_number"], int):
    fail("retry.attempt_number must be an integer")
if not isinstance(retry["payload_hash"], str):
    fail("retry.payload_hash must be a string")
if not isinstance(retry["previous_payload_hash"], str):
    fail("retry.previous_payload_hash must be a string")
if not isinstance(retry["duplicate_request"], bool):
    fail("retry.duplicate_request must be a boolean")

if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

if not isinstance(payload["decision_reasons"], list) or not all(
    isinstance(item, str) for item in payload["decision_reasons"]
):
    fail("decision_reasons must be an array of strings")

expected_go = True
if approvals["required"] <= 0:
    expected_go = False
if approvals["received"] < approvals["required"]:
    expected_go = False
if not approvals["approval_quorum_hash"].startswith("sha256:") or len(
    approvals["approval_quorum_hash"]
) <= len("sha256:"):
    expected_go = False
if not retry["idempotency_key"].startswith("idemp:") or len(retry["idempotency_key"]) <= len(
    "idemp:"
):
    expected_go = False
if retry["attempt_number"] < 1:
    expected_go = False
for hash_field in ("payload_hash", "previous_payload_hash"):
    hash_value = retry[hash_field]
    if not hash_value.startswith("sha256:") or len(hash_value) <= len("sha256:"):
        expected_go = False
if retry["attempt_number"] > 1 and retry["payload_hash"] != retry["previous_payload_hash"]:
    expected_go = False
if retry["duplicate_request"]:
    expected_go = False
if payload["ci_fast_gate"] != "PASS":
    expected_go = False

expected_decision = "GO" if expected_go else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")
if actual_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}"
    )

print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_decision}")
PY
)"

printf '%s\n' "$output"
