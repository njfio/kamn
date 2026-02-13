#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/task/run_federated_delegation_settlement_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
CONTRACT_MODULE="$ROOT_DIR/scripts/task/federated_delegation_settlement_contract_lane_contract.sh"
EXPECTED_MANIFEST="$ROOT_DIR/scripts/framework/manifests/task_federated_delegation_settlement_contract_lane.json"
DEEP_LANE="$ROOT_DIR/scripts/task/run_federated_delegation_settlement_deep_lane.sh"

if [ ! -x "$SCRIPT" ]; then
  echo "expected federated delegation settlement contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected non-Kolme dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

if [ ! -x "$CONTRACT_MODULE" ]; then
  echo "expected shared federated delegation settlement contract module to be executable: $CONTRACT_MODULE" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected federated delegation settlement wrapper to be a symlink: $SCRIPT" >&2
  exit 1
fi

manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$manifest_path" != "$EXPECTED_MANIFEST" ]; then
  echo "expected dispatcher to resolve $EXPECTED_MANIFEST but found $manifest_path" >&2
  exit 1
fi

if ! grep -Fq "\"scripts/task/$(basename "$CONTRACT_MODULE")\"" "$manifest_path"; then
  echo "expected manifest to dispatch shared federated delegation settlement contract module: $manifest_path" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected federated delegation settlement deep lane script to be executable" >&2
  exit 1
fi

if ! grep -Fq "run_federated_delegation_settlement_contract_lane.sh" "$DEEP_LANE"; then
  echo "expected federated delegation settlement deep lane script to invoke contract lane baseline checks first" >&2
  exit 1
fi

echo "federated delegation settlement contract lane wrapper tests passed."
