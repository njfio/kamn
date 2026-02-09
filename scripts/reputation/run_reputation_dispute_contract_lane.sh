#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/reputation/generate_reputation_dispute_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/reputation/check_reputation_dispute_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'EOF'
Usage:
  bash scripts/reputation/run_reputation_dispute_contract_lane.sh \
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
  fail "reputation dispute evidence generator is not executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "reputation dispute policy checker is not executable"
fi

if [ "$skip_tests" != true ]; then
  cargo test -p kamn-core --test reputation_state_model_docs >/dev/null
  cargo test -p kamn-core --test reputation_signal_routing_docs >/dev/null
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/reputation-dispute-contract.json"
fi

start_epoch="$(date +%s)"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --dispute-id "dispute-contract-001" \
    --subject-did "did:kamn:agent-contract-001" \
    --reviewer-did "did:kamn:reviewer-contract-001" \
    --dispute-reason-code "QUALITY" \
    --evidence-uri "s3://kamn-audit/reputation/dispute-contract-001.json" \
    --evidence-sha256 "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    --evidence-hash-verified "PASS" \
    --original-trust-score 620 \
    --proposed-trust-score 570 \
    --max-adjustment-points 90 \
    --policy-window-open true \
    --approval-recorded true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected reputation dispute contract lane bundle decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected reputation dispute contract lane policy decision to be GO" >&2
  exit 1
fi

if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  echo "expected reputation dispute contract lane to report no failed checks" >&2
  exit 1
fi

max_seconds="${REPUTATION_DISPUTE_MAX_SECONDS:-90}"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "reputation dispute contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'reason_key=%s\n' "$(extract_value "$generator_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
echo "reputation dispute contract lane tests passed."
