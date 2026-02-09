#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/escrow/check_settlement_reconciliation_evidence_policy.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/escrow/run_settlement_reconciliation_contract_lane.sh"

report_file="$ROOT_DIR/settlement-reconciliation-report.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      report_file="${2:-}"
      shift 2
      ;;
    --help|-h)
      cat <<'EOF'
Usage:
  bash scripts/escrow/run_settlement_reconciliation_deep_lane.sh [--output-json <path>]
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
    --escrow-id "escrow-deep-2026-02-09" \
    --settlement-outcome TIMEOUT_REFUNDED \
    --receipt-id "receipt-deep-pending" \
    --receipt-finality PENDING \
    --expected-release-amount 0 \
    --expected-refund-amount 120 \
    --observed-release-amount 0 \
    --observed-refund-amount 120 \
    --timeout-elapsed false \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected settlement reconciliation deep lane decision to be NO-GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$report_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected settlement reconciliation deep lane policy decision to be NO-GO" >&2
  exit 1
fi

echo "settlement reconciliation deep lane tests passed."
