#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/did/generate_lifecycle_operator_binding_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/did/check_lifecycle_operator_binding_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/did/run_lifecycle_operator_binding_contract_lane.sh \
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
  fail "lifecycle operator-binding evidence generator is not executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "lifecycle operator-binding policy checker is not executable"
fi

cd "$ROOT_DIR"
if [ "$skip_tests" != true ]; then
  cargo test -p kamn-core --test did_registry_transactions -- functional_lifecycle_rotate_mutation_updates_document_and_emits_allowed_reason_code -- --exact >/dev/null
  cargo test -p kamn-core --test did_registry_transactions -- regression_lifecycle_replayed_or_unauthorized_mutation_fails_closed -- --exact >/dev/null
  cargo test -p kamn-core --test key_lifecycle_audit_trails_docs >/dev/null
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/lifecycle-operator-binding-contract.json"
fi

start_epoch="$(date +%s)"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --did "kamn:did:agent:agent-contract-890" \
    --actor-did "kamn:did:human:operator-contract-890" \
    --required-operator-did "kamn:did:human:operator-contract-890" \
    --mutation-action "rotate" \
    --mutation-nonce 52 \
    --mutation-reason-code "did_lifecycle_mutation_allowed" \
    --audit-export-id "audit-export-contract-890" \
    --audit-record-count 2 \
    --audit-digest "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  fail "expected lifecycle operator-binding evidence generation decision to be GO"
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected lifecycle operator-binding policy decision to be GO"
fi
if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  fail "expected lifecycle operator-binding contract lane to report no failed checks"
fi

max_seconds="${DID_LIFECYCLE_OPERATOR_BINDING_MAX_SECONDS:-90}"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "lifecycle operator-binding contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'reason_key=%s\n' "$(extract_value "$generator_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
echo "lifecycle operator-binding contract lane tests passed."
