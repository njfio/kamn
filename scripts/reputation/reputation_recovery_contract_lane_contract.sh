#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/reputation/generate_reputation_recovery_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/reputation/check_reputation_recovery_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'EOF'
Usage:
  bash scripts/reputation/run_reputation_recovery_contract_lane.sh \
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
  fail "reputation recovery evidence generator is not executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "reputation recovery policy checker is not executable"
fi

if [ "$skip_tests" != true ]; then
  cargo test -p kamn-core --test reputation_state_model_docs >/dev/null
  cargo test -p kamn-core --test reputation_signal_routing_docs >/dev/null
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/reputation-recovery-contract.json"
fi

start_epoch="$(date +%s)"

generation_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --lane contract \
    --recovery-id "recovery-contract-001" \
    --subject-did "did:kamn:agent-contract-001" \
    --reviewer-did "did:kamn:reviewer-contract-001" \
    --pre-penalty-trust-score 710 \
    --post-penalty-trust-score 550 \
    --proposed-recovered-trust-score 670 \
    --max-reversal-points 180 \
    --false-positive-confirmed true \
    --reviewer-quorum-satisfied true \
    --audit-evidence-verified PASS \
    --replay-guard-pass true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generation_output" | grep -q "^final_decision=GO$"; then
  fail "expected reputation recovery contract lane bundle decision to be GO"
fi

if ! printf '%s\n' "$generation_output" | grep -q "^recovery_action=REVERSE_PENALTY$"; then
  fail "expected reputation recovery contract lane action to be REVERSE_PENALTY"
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected reputation recovery contract lane policy decision to be GO"
fi

if ! printf '%s\n' "$policy_output" | grep -q "^recovery_action=REVERSE_PENALTY$"; then
  fail "expected reputation recovery contract lane action to be REVERSE_PENALTY"
fi

if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  fail "expected reputation recovery contract lane to report no failed checks"
fi

max_seconds="${REPUTATION_RECOVERY_MAX_SECONDS:-90}"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "reputation recovery contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'reason_key=%s\n' "$(extract_value "$generation_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
printf 'recovery_action=%s\n' "$(extract_value "$policy_output" "recovery_action")"
echo "reputation recovery contract lane tests passed."
