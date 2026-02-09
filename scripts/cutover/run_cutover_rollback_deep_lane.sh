#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/cutover/generate_cutover_rollback_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/cutover/check_cutover_rollback_evidence_policy.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/cutover/run_cutover_rollback_contract_lane.sh"

report_file="$ROOT_DIR/cutover-rollback-report.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      report_file="${2:-}"
      shift 2
      ;;
    --help|-h)
      cat <<'EOF'
Usage:
  bash scripts/cutover/run_cutover_rollback_deep_lane.sh \
    [--output-json <path>]
EOF
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

mkdir -p "$(dirname "$report_file")"

bash "$CONTRACT_LANE"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$report_file" \
    --cutover-manifest-id "cutover-mainnet-deep-2026-02-09" \
    --rollback-trigger-status TRIGGERED \
    --checkpoint-state FAILED \
    --failed-checkpoint-id "" \
    --rollback-target-hash "state-hash-expected" \
    --post-rollback-hash "state-hash-observed" \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected rollback deep-lane mismatch scenario decision to be NO-GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$report_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected rollback deep-lane policy decision to be NO-GO" >&2
  exit 1
fi

echo "cutover rollback deep lane tests passed."
