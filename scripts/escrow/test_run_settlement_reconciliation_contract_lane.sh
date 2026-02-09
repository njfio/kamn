#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/escrow/run_settlement_reconciliation_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/escrow/run_settlement_reconciliation_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected settlement reconciliation fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected settlement reconciliation deep-lane runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "settlement reconciliation contract lane tests passed." "$TMP_OUT"; then
  echo "expected settlement reconciliation contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_settlement_reconciliation_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute settlement reconciliation fast-lane checks first" >&2
  exit 1
fi

if ! grep -q "settlement-reconciliation-report.json" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to emit settlement reconciliation report artifact" >&2
  exit 1
fi

if ! grep -q "final_decision=NO-GO" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to validate NO-GO settlement decision path" >&2
  exit 1
fi

if ! grep -q "run_settlement_reconciliation_race_matrix.py" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute settlement reconciliation race matrix checks" >&2
  exit 1
fi

if ! grep -q "fixtures/escrow_reconciliation/finality_race_cases.json" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to consume settlement race fixture matrix" >&2
  exit 1
fi

echo "settlement reconciliation contract lane script tests passed."
