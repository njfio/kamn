#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/task/generate_federated_delegation_settlement_evidence_bundle.sh \
    --output-file <path> \
    --delegation-id <value> \
    --task-id <value> \
    --delegator-did <value> \
    --delegatee-did <value> \
    --source-network <value> \
    --destination-network <value> \
    --settlement-reference-id <value> \
    --expected-settlement-reference-id <value> \
    --settlement-receipt-finality FINAL|PENDING|FAILED \
    --nonce-monotonic true|false \
    --replay-detected true|false \
    --partition-sequence-monotonic true|false \
    --required-attestors <integer> \
    --received-attestors <integer> \
    --ci-fast-gate PASS|FAIL
USAGE
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

require_non_negative_int() {
  local value="$1"
  local name="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "$name must be a non-negative integer"
  fi
}

output_file=""
delegation_id=""
task_id=""
delegator_did=""
delegatee_did=""
source_network=""
destination_network=""
settlement_reference_id=""
expected_settlement_reference_id=""
settlement_receipt_finality=""
nonce_monotonic=""
replay_detected=""
partition_sequence_monotonic=""
required_attestors=""
received_attestors=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --delegation-id)
      delegation_id="${2:-}"
      shift 2
      ;;
    --task-id)
      task_id="${2:-}"
      shift 2
      ;;
    --delegator-did)
      delegator_did="${2:-}"
      shift 2
      ;;
    --delegatee-did)
      delegatee_did="${2:-}"
      shift 2
      ;;
    --source-network)
      source_network="${2:-}"
      shift 2
      ;;
    --destination-network)
      destination_network="${2:-}"
      shift 2
      ;;
    --settlement-reference-id)
      settlement_reference_id="${2:-}"
      shift 2
      ;;
    --expected-settlement-reference-id)
      expected_settlement_reference_id="${2:-}"
      shift 2
      ;;
    --settlement-receipt-finality)
      settlement_receipt_finality="${2:-}"
      shift 2
      ;;
    --nonce-monotonic)
      nonce_monotonic="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --replay-detected)
      replay_detected="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --partition-sequence-monotonic)
      partition_sequence_monotonic="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --required-attestors)
      required_attestors="${2:-}"
      shift 2
      ;;
    --received-attestors)
      received_attestors="${2:-}"
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

if [[ -z "$output_file" || -z "$delegation_id" || -z "$task_id" || -z "$delegator_did" || -z "$delegatee_did" || -z "$source_network" || -z "$destination_network" || -z "$settlement_reference_id" || -z "$expected_settlement_reference_id" || -z "$settlement_receipt_finality" || -z "$nonce_monotonic" || -z "$replay_detected" || -z "$partition_sequence_monotonic" || -z "$required_attestors" || -z "$received_attestors" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all delegation settlement bundle arguments are required"
fi

if [[ "$settlement_receipt_finality" != "FINAL" && "$settlement_receipt_finality" != "PENDING" && "$settlement_receipt_finality" != "FAILED" ]]; then
  fail "settlement-receipt-finality must be FINAL, PENDING, or FAILED"
fi

if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  fail "ci-fast-gate must be PASS or FAIL"
fi

require_non_negative_int "$required_attestors" "required-attestors"
require_non_negative_int "$received_attestors" "received-attestors"

if [[ "$required_attestors" -eq 0 ]]; then
  fail "required-attestors must be greater than zero"
fi

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$delegation_id" "$task_id" "$delegator_did" "$delegatee_did" "$source_network" "$destination_network" "$settlement_reference_id" "$expected_settlement_reference_id" "$settlement_receipt_finality" "$nonce_monotonic" "$replay_detected" "$partition_sequence_monotonic" "$required_attestors" "$received_attestors" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys

(
    output_file,
    generated_at,
    delegation_id,
    task_id,
    delegator_did,
    delegatee_did,
    source_network,
    destination_network,
    settlement_reference_id,
    expected_settlement_reference_id,
    settlement_receipt_finality,
    nonce_monotonic_raw,
    replay_detected_raw,
    partition_sequence_monotonic_raw,
    required_attestors_raw,
    received_attestors_raw,
    ci_fast_gate,
) = sys.argv[1:]

nonce_monotonic = nonce_monotonic_raw == "true"
replay_detected = replay_detected_raw == "true"
partition_sequence_monotonic = partition_sequence_monotonic_raw == "true"
required_attestors = int(required_attestors_raw)
received_attestors = int(received_attestors_raw)

settlement_reference_present = bool(settlement_reference_id.strip()) and bool(
    expected_settlement_reference_id.strip()
)
settlement_reference_match = settlement_reference_id == expected_settlement_reference_id
receipt_finality_final = settlement_receipt_finality == "FINAL"
quorum_satisfied = received_attestors >= required_attestors
replay_guard_passed = (
    nonce_monotonic and not replay_detected and partition_sequence_monotonic
)
cross_network_delegation = (
    bool(source_network.strip())
    and bool(destination_network.strip())
    and source_network != destination_network
)
delegation_context_present = all(
    bool(value.strip())
    for value in (delegation_id, task_id, delegator_did, delegatee_did)
)

is_go = (
    delegation_context_present
    and settlement_reference_present
    and settlement_reference_match
    and receipt_finality_final
    and replay_guard_passed
    and quorum_satisfied
    and cross_network_delegation
    and ci_fast_gate == "PASS"
)

reason_codes: list[str] = []
if not delegation_context_present:
    reason_codes.append("delegation_context_missing")
if not settlement_reference_present:
    reason_codes.append("settlement_reference_missing")
if settlement_reference_present and not settlement_reference_match:
    reason_codes.append("settlement_reference_drift")
if not receipt_finality_final:
    reason_codes.append("settlement_receipt_not_final")
if not nonce_monotonic:
    reason_codes.append("nonce_not_monotonic")
if replay_detected:
    reason_codes.append("replay_detected")
if not partition_sequence_monotonic:
    reason_codes.append("partition_sequence_replayed")
if not quorum_satisfied:
    reason_codes.append("attestor_quorum_shortfall")
if not cross_network_delegation:
    reason_codes.append("non_federated_network_pair")
if ci_fast_gate != "PASS":
    reason_codes.append("ci_fast_gate_failed")

final_decision = "GO" if is_go else "NO-GO"

payload = {
    "schema_version": "kamn.task.federated-delegation-settlement.v1",
    "generated_at": generated_at,
    "delegation_id": delegation_id,
    "task_id": task_id,
    "delegator_did": delegator_did,
    "delegatee_did": delegatee_did,
    "source_network": source_network,
    "destination_network": destination_network,
    "settlement_reference_id": settlement_reference_id,
    "expected_settlement_reference_id": expected_settlement_reference_id,
    "settlement_receipt_finality": settlement_receipt_finality,
    "nonce_monotonic": nonce_monotonic,
    "replay_detected": replay_detected,
    "partition_sequence_monotonic": partition_sequence_monotonic,
    "required_attestors": required_attestors,
    "received_attestors": received_attestors,
    "ci_fast_gate": ci_fast_gate,
    "policy_checks": {
        "delegation_context_present": delegation_context_present,
        "settlement_reference_present": settlement_reference_present,
        "settlement_reference_match": settlement_reference_match,
        "receipt_finality_final": receipt_finality_final,
        "replay_guard_passed": replay_guard_passed,
        "quorum_satisfied": quorum_satisfied,
        "cross_network_delegation": cross_network_delegation,
    },
    "reason_codes": reason_codes,
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
