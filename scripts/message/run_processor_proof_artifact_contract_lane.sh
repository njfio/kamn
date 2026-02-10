#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/message/generate_processor_proof_artifact_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/message/check_processor_proof_artifact_policy.sh"
ZK_DOC="$ROOT_DIR/docs/foundation/zk-message-proof-design.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/message/run_processor_proof_artifact_contract_lane.sh \
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
  fail "processor proof artifact evidence generator is not executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "processor proof artifact policy checker is not executable"
fi

if [ ! -f "$ZK_DOC" ]; then
  fail "expected zk message proof design doc to exist"
fi

cd "$ROOT_DIR"
if [ "$skip_tests" != true ]; then
  bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test zk_message_proofs zk_message_proofs_regression_rejects_malformed_private_field_selector -- --exact
  bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test zk_message_proofs zk_message_proofs_reject_invalid_processor_artifact_commitment_shape -- --exact
  bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test zk_message_proofs zk_message_proofs_reject_invalid_processor_artifact_proof_value_shape -- --exact
  bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test zk_message_proofs_docs regression_requires_witness_artifact_contract_lane_marker -- --exact
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/processor-proof-artifact-contract.json"
fi

start_epoch="$(date +%s)"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --artifact-id "artifact-zk-993" \
    --message-id "urn:uuid:zk-msg-993" \
    --payload-commitment "fnv1a64:9b4f3e178aa01234" \
    --proof-value "proof:ok:artifact-zk-993" \
    --private-selector "task.description" \
    --private-selector "task.type" \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  fail "expected processor proof artifact evidence generation decision to be GO"
fi
if ! printf '%s\n' "$generator_output" | grep -q "^reason_key=zk_processor_proof_artifact_reason_codes:GO:v1$"; then
  fail "expected processor proof artifact GO reason key marker"
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected processor proof artifact policy decision to be GO"
fi
if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  fail "expected processor proof artifact contract lane to report failed_checks=none"
fi

if ! grep -Fq "run_processor_proof_artifact_contract_lane.sh" "$ZK_DOC"; then
  fail "expected zk message proof design doc to reference processor proof artifact contract lane"
fi
if ! grep -Fq "Regression: #993" "$ZK_DOC"; then
  fail "expected zk message proof design doc to include witness/artifact schema regression marker"
fi

max_seconds="${PROCESSOR_PROOF_ARTIFACT_MAX_SECONDS:-90}"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "processor proof artifact contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'reason_key=%s\n' "$(extract_value "$policy_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
echo "processor proof artifact contract lane tests passed."
