#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_SCRIPT="$ROOT_DIR/scripts/reputation/run_weighted_decay_matrix.py"
COMPACT_FIXTURE="$ROOT_DIR/fixtures/reputation_decay/compact_cases.json"
ADVERSARIAL_FIXTURE="$ROOT_DIR/fixtures/reputation_decay/adversarial_cases.json"
GENERATOR="$ROOT_DIR/scripts/reputation/generate_weighted_decay_property_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/reputation/check_weighted_decay_property_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'EOF'
Usage:
  bash scripts/reputation/run_weighted_decay_contract_lane.sh \
    [--output-file <path>] \
    [--skip-tests]
EOF
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
  fail "weighted decay property evidence generator is not executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "weighted decay property policy checker is not executable"
fi

if [ "$skip_tests" != true ]; then
  cargo test -p kamn-core --test trust_score_engine >/dev/null
  cargo test -p kamn-core --test trust_score_property_invariants >/dev/null
  cargo test -p kamn-core --test trust_score_engine_docs >/dev/null
fi

start_epoch="$(date +%s)"
compact_report_json="$TMP_DIR/reputation-weighted-decay-compact-contract-report.json"
adversarial_report_json="$TMP_DIR/reputation-weighted-decay-adversarial-contract-report.json"

matrix_output="$(
  python3 "$MATRIX_SCRIPT" \
    --fixture "$COMPACT_FIXTURE" \
    --output-json "$compact_report_json"
)"

if ! printf '%s\n' "$matrix_output" | grep -q '^status=pass;'; then
  echo "expected weighted decay compact matrix to pass" >&2
  exit 1
fi

adversarial_matrix_output="$(
  python3 "$MATRIX_SCRIPT" \
    --fixture "$ADVERSARIAL_FIXTURE" \
    --output-json "$adversarial_report_json"
)"

if ! printf '%s\n' "$adversarial_matrix_output" | grep -q '^status=pass;'; then
  echo "expected weighted decay adversarial matrix to pass" >&2
  exit 1
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/reputation-weighted-decay-property-contract-bundle.json"
fi

max_seconds="${REPUTATION_WEIGHTED_DECAY_MAX_SECONDS:-90}"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"
runtime_budget_status="within"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  runtime_budget_status="exceeded"
fi

generation_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --lane contract \
    --compact-report-file "$compact_report_json" \
    --adversarial-report-file "$adversarial_report_json" \
    --property-suite-status pass \
    --runtime-budget-status "$runtime_budget_status" \
    --ci-fast-gate PASS
)"

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"
policy_decision="$(extract_value "$policy_output" "final_decision")"

if [ "$runtime_budget_status" = "exceeded" ]; then
  if [ "$policy_decision" != "NO-GO" ]; then
    fail "expected NO-GO weighted decay policy decision when runtime budget is exceeded"
  fi
  fail "weighted decay contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

if [ "$policy_decision" != "GO" ]; then
  fail "expected GO weighted decay policy decision for contract lane"
fi

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'schema_version=%s\n' "$(extract_value "$policy_output" "schema_version")"
printf 'evidence_key=%s\n' "$(extract_value "$generation_output" "evidence_key")"
printf 'reason_key=%s\n' "$(extract_value "$generation_output" "reason_key")"
printf 'final_decision=%s\n' "$policy_decision"
echo "weighted decay contract lane tests passed."
