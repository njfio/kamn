#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"
WRAPPER_NAME="run_deployment_slo_rollback_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
CONTRACT_MODULE="$ROOT_DIR/scripts/deploy/deployment_slo_rollback_contract_lane_contract.sh"
EXPECTED_MANIFEST="$ROOT_DIR/scripts/framework/manifests/deploy_deployment_slo_rollback_contract_lane.json"

if [ ! -x "$MANIFEST_RUNNER" ]; then
  echo "expected manifest runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected non-Kolme dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

if [ ! -x "$CONTRACT_MODULE" ]; then
  echo "expected shared deployment slo/rollback contract module to be executable: $CONTRACT_MODULE" >&2
  exit 1
fi

manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$WRAPPER_NAME" --resolve-manifest-path)"
if [ "$manifest_path" != "$EXPECTED_MANIFEST" ]; then
  echo "expected dispatcher to resolve $EXPECTED_MANIFEST but found $manifest_path" >&2
  exit 1
fi

if ! grep -Fq "\"scripts/deploy/$(basename "$CONTRACT_MODULE")\"" "$manifest_path"; then
  echo "expected manifest to dispatch shared deployment slo/rollback contract module: $manifest_path" >&2
  exit 1
fi

if ! grep -q 'check_deployment_slo_rollback_policy.sh' "$CONTRACT_MODULE"; then
  echo "expected deployment slo/rollback contract module to execute policy checker" >&2
  exit 1
fi

bash "$MANIFEST_RUNNER" --manifest "$EXPECTED_MANIFEST" --phase contract >/dev/null

echo "deployment slo/rollback contract lane wrapper tests passed."
