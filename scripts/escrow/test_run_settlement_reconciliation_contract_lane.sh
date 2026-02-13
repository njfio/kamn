#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/escrow/run_settlement_reconciliation_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
CONTRACT_MODULE="$ROOT_DIR/scripts/escrow/settlement_reconciliation_contract_lane_contract.sh"
EXPECTED_MANIFEST="$ROOT_DIR/scripts/framework/manifests/escrow_settlement_reconciliation_contract_lane.json"
DEEP_SCRIPT="$ROOT_DIR/scripts/escrow/run_settlement_reconciliation_deep_lane.sh"

if [ ! -x "$SCRIPT" ]; then
  echo "expected settlement reconciliation fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected non-Kolme dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

if [ ! -x "$CONTRACT_MODULE" ]; then
  echo "expected shared settlement reconciliation contract module to be executable: $CONTRACT_MODULE" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected settlement reconciliation wrapper to be a symlink: $SCRIPT" >&2
  exit 1
fi

manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$manifest_path" != "$EXPECTED_MANIFEST" ]; then
  echo "expected dispatcher to resolve $EXPECTED_MANIFEST but found $manifest_path" >&2
  exit 1
fi

if ! grep -Fq "\"scripts/escrow/$(basename "$CONTRACT_MODULE")\"" "$manifest_path"; then
  echo "expected manifest to dispatch shared settlement reconciliation contract module: $manifest_path" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected settlement reconciliation deep-lane runner to be executable" >&2
  exit 1
fi

if ! grep -Fq "run_settlement_reconciliation_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute settlement reconciliation fast-lane checks first" >&2
  exit 1
fi

echo "settlement reconciliation contract lane wrapper tests passed."
