#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/signer/generate_secure_provider_key_lifecycle_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/signer/check_secure_provider_key_lifecycle_policy.sh"
KEY_LIFECYCLE_DOC="$ROOT_DIR/docs/foundation/key-lifecycle-audit-trails.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/signer/run_secure_provider_key_lifecycle_contract_lane.sh \
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
  fail "secure-provider key-lifecycle evidence generator is not executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "secure-provider key-lifecycle policy checker is not executable"
fi

if [ ! -f "$KEY_LIFECYCLE_DOC" ]; then
  fail "expected key lifecycle audit trails doc to exist"
fi

cd "$ROOT_DIR"
if [ "$skip_tests" != true ]; then
  bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test signer_backend integration_aws_kms_signed_transaction_passes_transaction_guards -- --exact
  bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test key_lifecycle_audit_trails_docs
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/secure-provider-key-lifecycle-contract.json"
fi

start_epoch="$(date +%s)"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --secure-key-reference "secure:aws-kms:role-operator/key-ops-rotation-988" \
    --provider "aws-kms" \
    --key-role "operator" \
    --lifecycle-action "rotate" \
    --previous-version 8 \
    --target-version 9 \
    --incident-ticket "INC-5988" \
    --revocation-reason-code "operator-requested" \
    --required-approvals 2 \
    --received-approvals 2 \
    --custody-attestation-hash "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    --approval-quorum-hash "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  fail "expected secure-provider key-lifecycle evidence generation decision to be GO"
fi
if ! printf '%s\n' "$generator_output" | grep -q "^reason_key=secure_provider_key_lifecycle_reason_codes:GO:v1$"; then
  fail "expected secure-provider key-lifecycle GO reason key marker"
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected secure-provider key-lifecycle policy decision to be GO"
fi
if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  fail "expected secure-provider key-lifecycle contract lane to report failed_checks=none"
fi

if ! grep -Fq "run_secure_provider_key_lifecycle_contract_lane.sh" "$KEY_LIFECYCLE_DOC"; then
  fail "expected key lifecycle audit trails doc to reference secure-provider key-lifecycle contract lane"
fi
if ! grep -Fq "Regression: #988" "$KEY_LIFECYCLE_DOC"; then
  fail "expected key lifecycle audit trails doc to include secure-provider key-lifecycle regression marker"
fi

max_seconds="${SIGNER_KEY_LIFECYCLE_MAX_SECONDS:-90}"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "secure-provider key-lifecycle contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'reason_key=%s\n' "$(extract_value "$policy_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
echo "secure-provider key-lifecycle contract lane tests passed."
