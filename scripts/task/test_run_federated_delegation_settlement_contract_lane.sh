#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/task/run_federated_delegation_settlement_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
CONTRACT_MODULE="$ROOT_DIR/scripts/task/federated_delegation_settlement_contract_lane_contract.sh"
EXPECTED_MANIFEST="$ROOT_DIR/scripts/framework/manifests/task_federated_delegation_settlement_contract_lane.json"
DEEP_LANE="$ROOT_DIR/scripts/task/run_federated_delegation_settlement_deep_lane.sh"
DEEP_IMPL="$ROOT_DIR/scripts/task/run_federated_delegation_settlement_deep_lane_impl.sh"
EXPECTED_DEEP_MANIFEST="$ROOT_DIR/scripts/framework/manifests/task_federated_delegation_settlement_deep_lane.json"

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
if [ ! -x "$DEEP_IMPL" ]; then
  echo "expected federated delegation settlement deep lane implementation module to be executable" >&2
  exit 1
fi

if [ ! -L "$DEEP_LANE" ]; then
  echo "expected federated delegation settlement deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$DEEP_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected federated delegation settlement deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_deep_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_LANE")" --resolve-manifest-path)"
if [ "$resolved_deep_manifest" != "$EXPECTED_DEEP_MANIFEST" ]; then
  echo "expected federated delegation settlement deep lane wrapper to resolve deep manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "\"scripts/task/$(basename "$DEEP_IMPL")\"" "$resolved_deep_manifest"; then
  echo "expected deep manifest to dispatch shared federated delegation settlement deep module: $resolved_deep_manifest" >&2
  exit 1
fi

echo "federated delegation settlement contract lane wrapper tests passed."
