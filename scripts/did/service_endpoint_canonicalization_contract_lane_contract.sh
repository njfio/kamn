#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/did/generate_service_endpoint_canonicalization_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/did/check_service_endpoint_canonicalization_policy.sh"
MATRIX_RUNNER="$ROOT_DIR/scripts/did/run_service_endpoint_canonicalization_matrix.py"
FIXTURE="$ROOT_DIR/fixtures/did_core_conformance/service_endpoint_canonicalization_vectors.json"
DID_CORE_DOC="$ROOT_DIR/docs/foundation/did-core-conformance-kamn-method.md"
DID_METHOD_DOC="$ROOT_DIR/docs/foundation/did-method.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/did/run_service_endpoint_canonicalization_contract_lane.sh \
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
  fail "service endpoint canonicalization evidence generator is not executable"
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  fail "service endpoint canonicalization policy checker is not executable"
fi
if [ ! -x "$MATRIX_RUNNER" ]; then
  fail "service endpoint canonicalization matrix runner is not executable"
fi
if [ ! -f "$FIXTURE" ]; then
  fail "service endpoint canonicalization fixture file is missing"
fi
if [ ! -f "$DID_CORE_DOC" ]; then
  fail "expected DID core conformance doc to exist"
fi
if [ ! -f "$DID_METHOD_DOC" ]; then
  fail "expected DID method doc to exist"
fi

cd "$ROOT_DIR"
if [ "$skip_tests" != true ]; then
  cargo test -p kamn-core --test did_method unit_service_endpoint_canonicalization_normalizes_case_and_whitespace -- --exact >/dev/null
  cargo test -p kamn-core --test did_method functional_service_endpoint_canonicalization_rejects_non_kamn_scheme -- --exact >/dev/null
  cargo test -p kamn-core --test did_method regression_service_endpoint_canonicalization_rejects_non_single_segment_path -- --exact >/dev/null
  cargo test -p kamn-core --test did_core_conformance_docs profile_contains_service_endpoint_canonicalization_contract_lane -- --exact >/dev/null
  cargo test -p kamn-core --test did_method_docs regression_requires_service_endpoint_canonicalization_guard_marker -- --exact >/dev/null
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/did-service-endpoint-canonicalization-contract.json"
fi

start_epoch="$(date +%s)"

matrix_report="$TMP_DIR/did-service-endpoint-canonicalization-matrix-report.json"
matrix_output="$(
  python3 "$MATRIX_RUNNER" \
    --fixture "$FIXTURE" \
    --output-json "$matrix_report"
)"
if ! printf '%s\n' "$matrix_output" | grep -q "^final_decision=GO$"; then
  fail "expected service endpoint canonicalization matrix to produce GO decision"
fi

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --fixture "$FIXTURE" \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  fail "expected service endpoint canonicalization evidence generation decision to be GO"
fi
if ! printf '%s\n' "$generator_output" | grep -q "^reason_key=did_service_endpoint_canonicalization_reason_codes:GO:v1$"; then
  fail "expected service endpoint canonicalization GO reason key marker"
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected service endpoint canonicalization policy decision to be GO"
fi
if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  fail "expected service endpoint canonicalization contract lane to report failed_checks=none"
fi

if ! grep -Fq "run_service_endpoint_canonicalization_contract_lane.sh" "$DID_CORE_DOC"; then
  fail "expected DID core conformance doc to reference service endpoint canonicalization contract lane"
fi
if ! grep -Fq "generate_service_endpoint_canonicalization_evidence_bundle.sh" "$DID_CORE_DOC"; then
  fail "expected DID core conformance doc to reference canonicalization evidence generator"
fi
if ! grep -Fq "check_service_endpoint_canonicalization_policy.sh" "$DID_CORE_DOC"; then
  fail "expected DID core conformance doc to reference canonicalization policy checker"
fi
if ! grep -Fq "run_service_endpoint_canonicalization_matrix.py" "$DID_CORE_DOC"; then
  fail "expected DID core conformance doc to reference canonicalization matrix runner"
fi
if ! grep -Fq "Regression: #1000" "$DID_CORE_DOC"; then
  fail "expected DID core conformance doc to include canonicalization regression marker"
fi
if ! grep -Fq "Regression: #1000" "$DID_METHOD_DOC"; then
  fail "expected DID method doc to include canonicalization regression marker"
fi

max_seconds="${DID_SERVICE_ENDPOINT_CANONICALIZATION_MAX_SECONDS:-90}"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "service endpoint canonicalization contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'reason_key=%s\n' "$(extract_value "$policy_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
echo "service endpoint canonicalization contract lane tests passed."
