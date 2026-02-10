#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/runtime/generate_processor_proof_admission_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_processor_proof_admission_policy.sh"
ZK_DOC="$ROOT_DIR/docs/foundation/zk-message-proof-design.md"
RUNTIME_DOC="$ROOT_DIR/docs/foundation/runtime-network.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/runtime/run_processor_proof_admission_contract_lane.sh \
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
  fail "processor proof admission evidence generator is not executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "processor proof admission policy checker is not executable"
fi

if [ ! -f "$ZK_DOC" ]; then
  fail "expected zk message proof design doc to exist"
fi

if [ ! -f "$RUNTIME_DOC" ]; then
  fail "expected runtime network doc to exist"
fi

cd "$ROOT_DIR"
if [ "$skip_tests" != true ]; then
  cargo test -p kamn-core --test message_lifecycle_proof_admission lifecycle_functional_valid_processor_proof_advances_to_validated -- --exact >/dev/null
  cargo test -p kamn-core --test message_lifecycle_proof_admission lifecycle_regression_tampered_proof_does_not_advance_validation_state -- --exact >/dev/null
  cargo test -p kamn-core --test message_lifecycle_proof_admission lifecycle_integration_replayed_artifact_is_rejected_for_second_message -- --exact >/dev/null
  cargo test -p kamn-core --test zk_message_proofs zk_message_proofs_reject_invalid_processor_artifact_proof_value_shape -- --exact >/dev/null
  cargo test -p kamn-core --test runtime_network_docs doc_contains_peer_lifecycle_and_queue_rules -- --exact >/dev/null
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/processor-proof-admission-contract.json"
fi

start_epoch="$(date +%s)"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --artifact-id "artifact-admission-995" \
    --message-id "urn:uuid:admission-995" \
    --message-id-match true \
    --commitment-match true \
    --proof-format-valid true \
    --replay-guard-active true \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  fail "expected processor proof admission evidence generation decision to be GO"
fi
if ! printf '%s\n' "$generator_output" | grep -q "^reason_key=processor_proof_admission_reason_codes:GO:v1$"; then
  fail "expected processor proof admission GO reason key marker"
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected processor proof admission policy decision to be GO"
fi
if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  fail "expected processor proof admission contract lane to report failed_checks=none"
fi

if ! grep -Fq "run_processor_proof_admission_contract_lane.sh" "$ZK_DOC"; then
  fail "expected zk message proof design doc to reference processor proof admission contract lane"
fi
if ! grep -Fq "run_processor_proof_admission_contract_lane.sh" "$RUNTIME_DOC"; then
  fail "expected runtime network doc to reference processor proof admission contract lane"
fi
if ! grep -Fq "Regression: #995" "$ZK_DOC"; then
  fail "expected zk message proof design doc to include processor admission regression marker"
fi
if ! grep -Fq "Regression: #995" "$RUNTIME_DOC"; then
  fail "expected runtime network doc to include processor admission regression marker"
fi

max_seconds="${PROCESSOR_PROOF_ADMISSION_MAX_SECONDS:-90}"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "processor proof admission contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'reason_key=%s\n' "$(extract_value "$policy_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
echo "processor proof admission contract lane tests passed."
