#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/did/generate_multikey_algorithm_policy_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/did/check_multikey_algorithm_policy.sh"
MATRIX_RUNNER="$ROOT_DIR/scripts/did/run_multikey_algorithm_migration_matrix.py"
FIXTURE="$ROOT_DIR/fixtures/did_core_conformance/multikey_algorithm_migration_vectors.json"
DID_CORE_DOC="$ROOT_DIR/docs/foundation/did-core-conformance-kamn-method.md"
DID_METHOD_DOC="$ROOT_DIR/docs/foundation/did-method.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/did/run_multikey_algorithm_policy_contract_lane.sh \
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
  fail "multikey algorithm policy evidence generator is not executable"
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  fail "multikey algorithm policy checker is not executable"
fi
if [ ! -x "$MATRIX_RUNNER" ]; then
  fail "multikey algorithm migration matrix runner is not executable"
fi
if [ ! -f "$FIXTURE" ]; then
  fail "multikey algorithm migration fixture file is missing"
fi
if [ ! -f "$DID_CORE_DOC" ]; then
  fail "expected DID core conformance doc to exist"
fi
if [ ! -f "$DID_METHOD_DOC" ]; then
  fail "expected DID method doc to exist"
fi

cd "$ROOT_DIR"
if [ "$skip_tests" != true ]; then
  cargo test -p kamn-core --test did_method unit_multikey_algorithm_policy_accepts_uniform_baseline_algorithms -- --exact >/dev/null
  cargo test -p kamn-core --test did_method functional_multikey_algorithm_policy_rejects_unsupported_algorithm -- --exact >/dev/null
  cargo test -p kamn-core --test did_method regression_multikey_algorithm_policy_rejects_mixed_algorithm_sets -- --exact >/dev/null
  cargo test -p kamn-core --test did_core_conformance_docs profile_contains_multikey_algorithm_migration_contract_lane -- --exact >/dev/null
  cargo test -p kamn-core --test did_method_docs regression_requires_multikey_algorithm_policy_guard_marker -- --exact >/dev/null
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/did-multikey-algorithm-policy-contract.json"
fi

start_epoch="$(date +%s)"

matrix_report="$TMP_DIR/did-multikey-algorithm-migration-matrix-report.json"
matrix_output="$(
  python3 "$MATRIX_RUNNER" \
    --fixture "$FIXTURE" \
    --output-json "$matrix_report"
)"
if ! printf '%s\n' "$matrix_output" | grep -q "^final_decision=GO$"; then
  fail "expected multikey algorithm migration matrix to produce GO decision"
fi

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --fixture "$FIXTURE" \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  fail "expected multikey algorithm policy evidence generation decision to be GO"
fi
if ! printf '%s\n' "$generator_output" | grep -q "^reason_key=did_multikey_algorithm_policy_reason_codes:GO:v1$"; then
  fail "expected multikey algorithm policy GO reason key marker"
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected multikey algorithm policy decision to be GO"
fi
if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  fail "expected multikey algorithm policy contract lane to report failed_checks=none"
fi

if ! grep -Fq "run_multikey_algorithm_policy_contract_lane.sh" "$DID_CORE_DOC"; then
  fail "expected DID core conformance doc to reference multikey algorithm policy contract lane"
fi
if ! grep -Fq "generate_multikey_algorithm_policy_evidence_bundle.sh" "$DID_CORE_DOC"; then
  fail "expected DID core conformance doc to reference multikey policy evidence generator"
fi
if ! grep -Fq "check_multikey_algorithm_policy.sh" "$DID_CORE_DOC"; then
  fail "expected DID core conformance doc to reference multikey policy checker"
fi
if ! grep -Fq "run_multikey_algorithm_migration_matrix.py" "$DID_CORE_DOC"; then
  fail "expected DID core conformance doc to reference multikey migration matrix runner"
fi
if ! grep -Fq "Regression: #1001" "$DID_CORE_DOC"; then
  fail "expected DID core conformance doc to include multikey algorithm regression marker"
fi
if ! grep -Fq "Regression: #1001" "$DID_METHOD_DOC"; then
  fail "expected DID method doc to include multikey algorithm regression marker"
fi

max_seconds="${DID_MULTIKEY_ALGORITHM_POLICY_MAX_SECONDS:-90}"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "multikey algorithm policy contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'reason_key=%s\n' "$(extract_value "$policy_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
echo "multikey algorithm policy contract lane tests passed."
