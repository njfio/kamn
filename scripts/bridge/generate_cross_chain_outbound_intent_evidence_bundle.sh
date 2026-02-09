#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/bridge/generate_cross_chain_outbound_intent_evidence_bundle.sh \
    --output-file <path> \
    --chain ethereum|near \
    --request-id <value> \
    --destination-channel <value> \
    --required-approvals <n> \
    --received-approvals <n> \
    --approval-quorum-hash <sha256:...> \
    --idempotency-key <idemp:...> \
    --attempt-number <n> \
    --payload-hash <sha256:...> \
    --previous-payload-hash <sha256:...> \
    --duplicate-request true|false \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

require_int() {
  local name="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "${name} must be an integer"
  fi
}

output_file=""
chain=""
request_id=""
destination_channel=""
required_approvals=""
received_approvals=""
approval_quorum_hash=""
idempotency_key=""
attempt_number=""
payload_hash=""
previous_payload_hash=""
duplicate_request=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --chain)
      chain="${2:-}"
      shift 2
      ;;
    --request-id)
      request_id="${2:-}"
      shift 2
      ;;
    --destination-channel)
      destination_channel="${2:-}"
      shift 2
      ;;
    --required-approvals)
      required_approvals="${2:-}"
      shift 2
      ;;
    --received-approvals)
      received_approvals="${2:-}"
      shift 2
      ;;
    --approval-quorum-hash)
      approval_quorum_hash="${2:-}"
      shift 2
      ;;
    --idempotency-key)
      idempotency_key="${2:-}"
      shift 2
      ;;
    --attempt-number)
      attempt_number="${2:-}"
      shift 2
      ;;
    --payload-hash)
      payload_hash="${2:-}"
      shift 2
      ;;
    --previous-payload-hash)
      previous_payload_hash="${2:-}"
      shift 2
      ;;
    --duplicate-request)
      duplicate_request="${2:-}"
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

if [[ -z "$output_file" || -z "$chain" || -z "$request_id" || -z "$destination_channel" || -z "$required_approvals" || -z "$received_approvals" || -z "$approval_quorum_hash" || -z "$idempotency_key" || -z "$attempt_number" || -z "$payload_hash" || -z "$previous_payload_hash" || -z "$duplicate_request" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all cross-chain outbound intent evidence arguments are required"
fi

require_int "required-approvals" "$required_approvals"
require_int "received-approvals" "$received_approvals"
require_int "attempt-number" "$attempt_number"

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$chain" "$request_id" "$destination_channel" "$required_approvals" "$received_approvals" "$approval_quorum_hash" "$idempotency_key" "$attempt_number" "$payload_hash" "$previous_payload_hash" "$duplicate_request" "$ci_fast_gate" <<'PY'
import json
import pathlib
import re
import sys


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    chain,
    request_id,
    destination_channel,
    required_approvals_raw,
    received_approvals_raw,
    approval_quorum_hash,
    idempotency_key,
    attempt_number_raw,
    payload_hash,
    previous_payload_hash,
    duplicate_request_raw,
    ci_fast_gate,
) = sys.argv[1:]

if chain not in {"ethereum", "near"}:
    fail("chain must be ethereum or near")
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci-fast-gate must be PASS or FAIL")
if duplicate_request_raw not in {"true", "false"}:
    fail("duplicate-request must be true or false")

required_approvals = int(required_approvals_raw)
received_approvals = int(received_approvals_raw)
attempt_number = int(attempt_number_raw)
duplicate_request = duplicate_request_raw == "true"

decision_reasons: list[str] = []

if not request_id.strip():
    decision_reasons.append("request_id must not be empty")
if not destination_channel.startswith(f"{chain}:"):
    decision_reasons.append("destination channel must match selected chain prefix")
if required_approvals <= 0:
    decision_reasons.append("required approvals must be greater than zero")
if received_approvals < required_approvals:
    decision_reasons.append("received approvals are below required approvals")
if not approval_quorum_hash.startswith("sha256:") or len(approval_quorum_hash) <= len("sha256:"):
    decision_reasons.append("approval quorum hash must be a non-empty sha256 digest")
if not idempotency_key.startswith("idemp:") or len(idempotency_key) <= len("idemp:"):
    decision_reasons.append("idempotency key must use idemp:<value> format")
if attempt_number < 1:
    decision_reasons.append("attempt number must be at least 1")

for field_name, field_value in (
    ("payload_hash", payload_hash),
    ("previous_payload_hash", previous_payload_hash),
):
    if not field_value.startswith("sha256:") or len(field_value) <= len("sha256:"):
        decision_reasons.append(f"{field_name} must be a non-empty sha256 digest")

if attempt_number > 1 and payload_hash != previous_payload_hash:
    decision_reasons.append("retry payload hash drift detected")
if duplicate_request:
    decision_reasons.append("duplicate request replay detected")
if ci_fast_gate != "PASS":
    decision_reasons.append("ci-fast-gate-failed")

final_decision = "GO" if not decision_reasons else "NO-GO"
if not decision_reasons:
    decision_reasons.append("all outbound intent approval/idempotency gates satisfied")

payload = {
    "schema_version": "kamn.bridge.cross-chain-outbound-intent.v1",
    "generated_at": generated_at,
    "chain": chain,
    "request_id": request_id,
    "destination_channel": destination_channel,
    "approvals": {
        "required": required_approvals,
        "received": received_approvals,
        "approval_quorum_hash": approval_quorum_hash,
    },
    "retry": {
        "idempotency_key": idempotency_key,
        "attempt_number": attempt_number,
        "payload_hash": payload_hash,
        "previous_payload_hash": previous_payload_hash,
        "duplicate_request": duplicate_request,
    },
    "ci_fast_gate": ci_fast_gate,
    "decision_reasons": decision_reasons,
    "final_decision": final_decision,
}

path = pathlib.Path(output_file)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
print(final_decision)
PY
)"

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'final_decision=%s\n' "$final_decision"
