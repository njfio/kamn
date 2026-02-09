#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/did/generate_federated_did_handshake_evidence_bundle.sh \
    --output-file <path> \
    --handshake-id <value> \
    --subject-did <value> \
    --local-network <value> \
    --remote-network <value> \
    --resolver-cache-hit true|false \
    --resolver-version <value> \
    --signature-policy PASS|FAIL \
    --nonce-monotonic true|false \
    --downgrade-detected true|false \
    --partition-sequence-monotonic true|false \
    --required-quorum <integer> \
    --received-quorum <integer> \
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

require_positive_int() {
  local value="$1"
  local name="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "$name must be a non-negative integer"
  fi
}

output_file=""
handshake_id=""
subject_did=""
local_network=""
remote_network=""
resolver_cache_hit=""
resolver_version=""
signature_policy=""
nonce_monotonic=""
downgrade_detected=""
partition_sequence_monotonic=""
required_quorum=""
received_quorum=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --handshake-id)
      handshake_id="${2:-}"
      shift 2
      ;;
    --subject-did)
      subject_did="${2:-}"
      shift 2
      ;;
    --local-network)
      local_network="${2:-}"
      shift 2
      ;;
    --remote-network)
      remote_network="${2:-}"
      shift 2
      ;;
    --resolver-cache-hit)
      resolver_cache_hit="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --resolver-version)
      resolver_version="${2:-}"
      shift 2
      ;;
    --signature-policy)
      signature_policy="${2:-}"
      shift 2
      ;;
    --nonce-monotonic)
      nonce_monotonic="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --downgrade-detected)
      downgrade_detected="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --partition-sequence-monotonic)
      partition_sequence_monotonic="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --required-quorum)
      required_quorum="${2:-}"
      shift 2
      ;;
    --received-quorum)
      received_quorum="${2:-}"
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

if [[ -z "$output_file" || -z "$handshake_id" || -z "$subject_did" || -z "$local_network" || -z "$remote_network" || -z "$resolver_cache_hit" || -z "$resolver_version" || -z "$signature_policy" || -z "$nonce_monotonic" || -z "$downgrade_detected" || -z "$partition_sequence_monotonic" || -z "$required_quorum" || -z "$received_quorum" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all handshake bundle arguments are required"
fi

for status in "$signature_policy" "$ci_fast_gate"; do
  if [[ "$status" != "PASS" && "$status" != "FAIL" ]]; then
    fail "signature-policy and ci-fast-gate must be PASS or FAIL"
  fi
done

require_positive_int "$required_quorum" "required-quorum"
require_positive_int "$received_quorum" "received-quorum"

if [[ "$required_quorum" -eq 0 ]]; then
  fail "required-quorum must be greater than zero"
fi

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$handshake_id" "$subject_did" "$local_network" "$remote_network" "$resolver_cache_hit" "$resolver_version" "$signature_policy" "$nonce_monotonic" "$downgrade_detected" "$partition_sequence_monotonic" "$required_quorum" "$received_quorum" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys

(
    output_file,
    generated_at,
    handshake_id,
    subject_did,
    local_network,
    remote_network,
    resolver_cache_hit_raw,
    resolver_version,
    signature_policy,
    nonce_monotonic_raw,
    downgrade_detected_raw,
    partition_sequence_monotonic_raw,
    required_quorum_raw,
    received_quorum_raw,
    ci_fast_gate,
) = sys.argv[1:]

resolver_cache_hit = resolver_cache_hit_raw == "true"
nonce_monotonic = nonce_monotonic_raw == "true"
downgrade_detected = downgrade_detected_raw == "true"
partition_sequence_monotonic = partition_sequence_monotonic_raw == "true"
required_quorum = int(required_quorum_raw)
received_quorum = int(received_quorum_raw)

resolver_version_present = bool(resolver_version.strip())
signature_policy_passed = signature_policy == "PASS"
quorum_satisfied = received_quorum >= required_quorum
replay_guard_passed = nonce_monotonic and partition_sequence_monotonic
downgrade_guard_passed = not downgrade_detected

is_go = (
    resolver_version_present
    and signature_policy_passed
    and quorum_satisfied
    and replay_guard_passed
    and downgrade_guard_passed
    and ci_fast_gate == "PASS"
)

reason_codes = []
if not resolver_version_present:
    reason_codes.append("resolver_version_missing")
if not signature_policy_passed:
    reason_codes.append("signature_policy_failed")
if not quorum_satisfied:
    reason_codes.append("quorum_shortfall")
if not nonce_monotonic:
    reason_codes.append("nonce_replay_detected")
if not partition_sequence_monotonic:
    reason_codes.append("partition_sequence_replayed")
if downgrade_detected:
    reason_codes.append("downgrade_attack_detected")
if ci_fast_gate != "PASS":
    reason_codes.append("ci_fast_gate_failed")

final_decision = "GO" if is_go else "NO-GO"

payload = {
    "schema_version": "kamn.did.federated-handshake.v1",
    "generated_at": generated_at,
    "handshake_id": handshake_id,
    "subject_did": subject_did,
    "local_network": local_network,
    "remote_network": remote_network,
    "resolver_cache_hit": resolver_cache_hit,
    "resolver_version": resolver_version,
    "signature_policy": signature_policy,
    "nonce_monotonic": nonce_monotonic,
    "downgrade_detected": downgrade_detected,
    "partition_sequence_monotonic": partition_sequence_monotonic,
    "required_quorum": required_quorum,
    "received_quorum": received_quorum,
    "ci_fast_gate": ci_fast_gate,
    "policy_checks": {
        "resolver_version_present": resolver_version_present,
        "signature_policy_passed": signature_policy_passed,
        "quorum_satisfied": quorum_satisfied,
        "replay_guard_passed": replay_guard_passed,
        "downgrade_guard_passed": downgrade_guard_passed,
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
