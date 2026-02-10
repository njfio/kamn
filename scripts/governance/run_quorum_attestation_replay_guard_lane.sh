#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATOR_DOC="$ROOT_DIR/docs/foundation/validator-lifecycle-quorum-reconfiguration.md"
THREAT_DOC="$ROOT_DIR/docs/foundation/threat-control-matrix.md"

usage() {
  cat <<'EOF'
Usage:
  bash scripts/governance/run_quorum_attestation_replay_guard_lane.sh \
    [--output-file <path>]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

validate_bool_value() {
  local name="$1"
  local value="$2"
  case "$value" in
    true|false) ;;
    *)
      fail "${name} must be true or false"
      ;;
  esac
}

require_int() {
  local name="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "${name} must be an integer >= 0"
  fi
}

output_file="$ROOT_DIR/governance-quorum-attestation-replay-report.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
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

max_runtime_seconds="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_MAX_SECONDS:-180}"
require_int "KAMN_GOVERNANCE_QUORUM_ATTESTATION_MAX_SECONDS" "$max_runtime_seconds"

skip_commands="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_SKIP_COMMANDS:-false}"
validate_bool_value "KAMN_GOVERNANCE_QUORUM_ATTESTATION_SKIP_COMMANDS" "$skip_commands"

force_lane_failure="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_LANE_FAILURE:-false}"
force_missing_keys="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_MISSING_KEYS:-false}"
force_signature_metadata_invalid="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_SIGNATURE_METADATA_INVALID:-false}"
force_replay_detected="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_REPLAY_DETECTED:-false}"
force_approval_shortfall="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_APPROVAL_SHORTFALL:-false}"
force_docs_contract_missing="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_DOCS_CONTRACT_MISSING:-false}"

validate_bool_value "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_LANE_FAILURE" "$force_lane_failure"
validate_bool_value "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_MISSING_KEYS" "$force_missing_keys"
validate_bool_value "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_SIGNATURE_METADATA_INVALID" "$force_signature_metadata_invalid"
validate_bool_value "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_REPLAY_DETECTED" "$force_replay_detected"
validate_bool_value "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_APPROVAL_SHORTFALL" "$force_approval_shortfall"
validate_bool_value "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_DOCS_CONTRACT_MISSING" "$force_docs_contract_missing"

proposal_id="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_PROPOSAL_ID:-gov-quorum-attestation-001}"
approval_artifact_id="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_APPROVAL_ARTIFACT_ID:-approval-artifact-001}"
payload_hash="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_PAYLOAD_HASH:-sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa}"
approver_dids_csv="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_APPROVER_DIDS:-kamn:did:agent:validator-1,kamn:did:agent:validator-2}"
signature_algorithm="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_SIGNATURE_ALGORITHM:-ed25519}"
signature_key_id="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_SIGNATURE_KEY_ID:-governance-signing-key-001}"
signature_signed_at_unix="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_SIGNATURE_SIGNED_AT_UNIX:-1716305100}"
required_signatures="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_REQUIRED_SIGNATURES:-2}"
received_signatures="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_RECEIVED_SIGNATURES:-2}"
replay_detected="${KAMN_GOVERNANCE_QUORUM_ATTESTATION_REPLAY_DETECTED:-false}"

require_int "KAMN_GOVERNANCE_QUORUM_ATTESTATION_SIGNATURE_SIGNED_AT_UNIX" "$signature_signed_at_unix"
require_int "KAMN_GOVERNANCE_QUORUM_ATTESTATION_REQUIRED_SIGNATURES" "$required_signatures"
require_int "KAMN_GOVERNANCE_QUORUM_ATTESTATION_RECEIVED_SIGNATURES" "$received_signatures"
validate_bool_value "KAMN_GOVERNANCE_QUORUM_ATTESTATION_REPLAY_DETECTED" "$replay_detected"

if [[ "$force_approval_shortfall" == "true" ]]; then
  if [ "$required_signatures" -gt 0 ]; then
    received_signatures="$((required_signatures - 1))"
  else
    received_signatures="0"
  fi
fi

if [[ "$force_replay_detected" == "true" ]]; then
  replay_detected="true"
fi

if [[ "$force_signature_metadata_invalid" == "true" ]]; then
  signature_algorithm="legacy-rsa"
  signature_key_id=""
fi

if [[ "$force_missing_keys" == "true" ]]; then
  approval_artifact_id=""
  payload_hash=""
fi

commands=()
lane_failed=false
start_epoch="$(date +%s)"

if [[ "$skip_commands" != "true" ]]; then
  commands+=("cargo test -p kamn-core --test governance_workflow governance_workflow_functional_submit_vote_execute_flow")
  if ! cargo test -p kamn-core --test governance_workflow governance_workflow_functional_submit_vote_execute_flow >/dev/null; then
    lane_failed=true
  fi

  commands+=("cargo test -p kamn-core --test governance_workflow governance_workflow_regression_rejects_replayed_voter_approval_artifact")
  if ! cargo test -p kamn-core --test governance_workflow governance_workflow_regression_rejects_replayed_voter_approval_artifact >/dev/null; then
    lane_failed=true
  fi
fi

if [[ "$force_lane_failure" == "true" ]]; then
  lane_failed=true
fi

docs_contract_present=true
required_doc_markers=(
  "governance_quorum_attestation_replay_policy_contract.py"
  "run_quorum_attestation_replay_guard_lane.sh"
  "check_quorum_attestation_replay_policy.sh"
  "run_quorum_attestation_replay_contract_lane.sh"
  "kamn.governance.quorum-attestation-replay-report.v1"
  "governance_quorum_attestation_reason_codes:GO:v1"
  "governance_quorum_attestation_reason_codes:NO-GO:v1"
  'quorum attestation evidence drift and replay attempts must fail closed (`Regression: #911`).'
)

for marker in "${required_doc_markers[@]}"; do
  if ! grep -Fq "$marker" "$VALIDATOR_DOC"; then
    docs_contract_present=false
  fi
  if ! grep -Fq "$marker" "$THREAT_DOC"; then
    docs_contract_present=false
  fi
done

if [[ "$force_docs_contract_missing" == "true" ]]; then
  docs_contract_present=false
fi

required_keys_present=true
if [[ -z "$proposal_id" || -z "$approval_artifact_id" || -z "$payload_hash" || -z "$approver_dids_csv" ]]; then
  required_keys_present=false
fi
if [[ ! "$payload_hash" =~ ^sha256:[0-9a-f]{64}$ ]]; then
  required_keys_present=false
fi

IFS=',' read -r -a approver_dids <<<"$approver_dids_csv"
if [ "${#approver_dids[@]}" -eq 0 ]; then
  required_keys_present=false
fi
for did in "${approver_dids[@]}"; do
  if [[ -z "$did" || "$did" != kamn:did:agent:* ]]; then
    required_keys_present=false
  fi
done

signature_metadata_valid=true
case "$signature_algorithm" in
  ed25519|secp256k1) ;;
  *)
    signature_metadata_valid=false
    ;;
esac
if [[ -z "$signature_key_id" ]]; then
  signature_metadata_valid=false
fi
if [ "$signature_signed_at_unix" -le 0 ]; then
  signature_metadata_valid=false
fi

approval_quorum_met=true
if [ "$required_signatures" -lt 1 ] || [ "$received_signatures" -lt "$required_signatures" ]; then
  approval_quorum_met=false
fi

replay_guard_passed=true
if [[ "$replay_detected" == "true" ]]; then
  replay_guard_passed=false
fi

runtime_seconds="$(( $(date +%s) - start_epoch ))"
runtime_budget_ok=true
if [ "$runtime_seconds" -gt "$max_runtime_seconds" ]; then
  runtime_budget_ok=false
fi

decision_reasons=()
if [[ "$lane_failed" == "true" ]]; then
  decision_reasons+=("governance_quorum_lane_failed")
fi
if [[ "$required_keys_present" != "true" ]]; then
  decision_reasons+=("quorum_attestation_required_keys_missing")
fi
if [[ "$signature_metadata_valid" != "true" ]]; then
  decision_reasons+=("quorum_attestation_signature_metadata_invalid")
fi
if [[ "$approval_quorum_met" != "true" ]]; then
  decision_reasons+=("quorum_attestation_approval_quorum_missing")
fi
if [[ "$replay_guard_passed" != "true" ]]; then
  decision_reasons+=("quorum_attestation_replay_detected")
fi
if [[ "$docs_contract_present" != "true" ]]; then
  decision_reasons+=("quorum_attestation_docs_contract_missing")
fi
if [[ "$runtime_budget_ok" != "true" ]]; then
  decision_reasons+=("runtime_budget_exceeded")
fi

final_decision="GO"
if [ "${#decision_reasons[@]}" -gt 0 ]; then
  final_decision="NO-GO"
fi
reason_key="governance_quorum_attestation_reason_codes:${final_decision}:v1"

mkdir -p "$(dirname "$output_file")"
decision_reasons_json="$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1:]))' "${decision_reasons[@]}")"
commands_json="$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1:]))' "${commands[@]}")"
approver_dids_json="$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1:]))' "${approver_dids[@]}")"

python3 - "$output_file" "$max_runtime_seconds" "$runtime_seconds" "$proposal_id" "$approval_artifact_id" "$payload_hash" "$approver_dids_json" "$required_signatures" "$received_signatures" "$replay_detected" "$signature_algorithm" "$signature_key_id" "$signature_signed_at_unix" "$lane_failed" "$required_keys_present" "$signature_metadata_valid" "$approval_quorum_met" "$replay_guard_passed" "$docs_contract_present" "$runtime_budget_ok" "$decision_reasons_json" "$commands_json" "$final_decision" "$reason_key" <<'PY'
import json
import pathlib
import sys
from datetime import datetime, timezone

output_file = pathlib.Path(sys.argv[1])
max_runtime_seconds = int(sys.argv[2])
runtime_seconds = int(sys.argv[3])
proposal_id = sys.argv[4]
approval_artifact_id = sys.argv[5]
payload_hash = sys.argv[6]
approver_dids = json.loads(sys.argv[7])
required_signatures = int(sys.argv[8])
received_signatures = int(sys.argv[9])
replay_detected = sys.argv[10] == "true"
signature_algorithm = sys.argv[11]
signature_key_id = sys.argv[12]
signature_signed_at_unix = int(sys.argv[13])
lane_failed = sys.argv[14] == "true"
required_keys_present = sys.argv[15] == "true"
signature_metadata_valid = sys.argv[16] == "true"
approval_quorum_met = sys.argv[17] == "true"
replay_guard_passed = sys.argv[18] == "true"
docs_contract_present = sys.argv[19] == "true"
runtime_budget_ok = sys.argv[20] == "true"
decision_reasons = json.loads(sys.argv[21])
commands = json.loads(sys.argv[22])
final_decision = sys.argv[23]
reason_key = sys.argv[24]

payload = {
    "schema_version": "kamn.governance.quorum-attestation-replay-report.v1",
    "generated_at": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "max_runtime_seconds": max_runtime_seconds,
    "runtime_seconds": runtime_seconds,
    "attestation_bundle": {
        "proposal_id": proposal_id,
        "approval_artifact_id": approval_artifact_id,
        "payload_hash": payload_hash,
        "approver_dids": approver_dids,
        "required_signatures": required_signatures,
        "received_signatures": received_signatures,
        "replay_detected": replay_detected,
        "signature_metadata": {
            "algorithm": signature_algorithm,
            "key_id": signature_key_id,
            "signed_at_unix": signature_signed_at_unix,
        },
    },
    "checks": {
        "lane_failed": lane_failed,
        "required_keys_present": required_keys_present,
        "signature_metadata_valid": signature_metadata_valid,
        "approval_quorum_met": approval_quorum_met,
        "replay_guard_passed": replay_guard_passed,
        "docs_contract_present": docs_contract_present,
        "runtime_budget_ok": runtime_budget_ok,
    },
    "commands": commands,
    "decision_reasons": decision_reasons,
    "final_decision": final_decision,
    "reason_key": reason_key,
}
output_file.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

printf 'status=ok\n'
printf 'output_file=%s\n' "$output_file"
printf 'final_decision=%s\n' "$final_decision"
printf 'reason_key=%s\n' "$reason_key"
printf 'runtime_seconds=%s\n' "$runtime_seconds"
printf 'max_runtime_seconds=%s\n' "$max_runtime_seconds"
