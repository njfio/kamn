#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/runtime/generate_watchdog_proof_consensus_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_watchdog_proof_consensus_policy.sh"
WATCHDOG_DOC="$ROOT_DIR/docs/foundation/runtime-watchdog-attestation.md"
GONOGO_DOC="$ROOT_DIR/docs/foundation/release-gonogo-checklist.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/runtime/run_watchdog_proof_consensus_contract_lane.sh \
    [--output-file <path>] \
    [--skip-tests]
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

output_file=""
skip_tests=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --skip-tests)
      skip_tests=true
      shift
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

if [ ! -x "$GENERATOR" ]; then
  fail "watchdog proof consensus evidence generator is not executable"
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  fail "watchdog proof consensus policy checker is not executable"
fi
if [ ! -f "$WATCHDOG_DOC" ]; then
  fail "expected runtime watchdog attestation doc to exist"
fi
if [ ! -f "$GONOGO_DOC" ]; then
  fail "expected release go/no-go checklist doc to exist"
fi

cd "$ROOT_DIR"
if [ "$skip_tests" != true ]; then
  cargo test -p kamn-core --test zk_message_proofs zk_message_proofs_functional_validator_quorum_consensus_aligned_valid_is_nominal -- --exact >/dev/null
  cargo test -p kamn-core --test zk_message_proofs zk_message_proofs_integration_rejects_replayed_validator_attestation_id -- --exact >/dev/null
  cargo test -p kamn-core --test zk_message_proofs zk_message_proofs_regression_projects_validator_invalid_mismatch_to_watchdog_signal -- --exact >/dev/null
  cargo test -p kamn-core --test runtime_watchdog_attestation_docs doc_contains_validator_watchdog_proof_consensus_deep_lane_contract -- --exact >/dev/null
  cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_watchdog_proof_consensus_evidence_contract -- --exact >/dev/null
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/watchdog-proof-consensus-contract.json"
fi

start_epoch="$(date +%s)"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --message-id "urn:uuid:watchdog-proof-go-996" \
    --artifact-id "artifact-watchdog-go-996" \
    --consensus-status ConsensusValid \
    --required-quorum 2 \
    --valid-attestation-count 2 \
    --invalid-attestation-count 0 \
    --replay-attestation-count 0 \
    --cadence fast \
    --runtime-seconds 3 \
    --max-seconds 90 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  fail "expected watchdog proof consensus evidence generation decision to be GO"
fi
if ! printf '%s\n' "$generator_output" | grep -q "^reason_key=watchdog_proof_consensus_reason_codes:GO:v1$"; then
  fail "expected watchdog proof consensus GO reason key marker"
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected watchdog proof consensus policy decision to be GO"
fi
if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  fail "expected watchdog proof consensus contract lane to report failed_checks=none"
fi

if ! grep -Fq "run_watchdog_proof_consensus_contract_lane.sh" "$WATCHDOG_DOC"; then
  fail "expected runtime watchdog attestation doc to reference watchdog proof consensus contract lane"
fi
if ! grep -Fq "run_watchdog_proof_consensus_deep_lane.sh" "$WATCHDOG_DOC"; then
  fail "expected runtime watchdog attestation doc to reference watchdog proof consensus deep lane"
fi
if ! grep -Fq "Regression: #996" "$WATCHDOG_DOC"; then
  fail "expected runtime watchdog attestation doc to include watchdog proof consensus regression marker"
fi
if ! grep -Fq "run_watchdog_proof_consensus_contract_lane.sh" "$GONOGO_DOC"; then
  fail "expected release go/no-go checklist to reference watchdog proof consensus contract lane"
fi
if ! grep -Fq "run_watchdog_proof_consensus_deep_lane.sh" "$GONOGO_DOC"; then
  fail "expected release go/no-go checklist to reference watchdog proof consensus deep lane"
fi
if ! grep -Fq "Regression: #996" "$GONOGO_DOC"; then
  fail "expected release go/no-go checklist to include watchdog proof consensus regression marker"
fi

max_seconds="${KAMN_WATCHDOG_PROOF_CONSENSUS_MAX_SECONDS:-90}"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "watchdog proof consensus contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'reason_key=%s\n' "$(extract_value "$policy_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
echo "watchdog proof consensus contract lane tests passed."
