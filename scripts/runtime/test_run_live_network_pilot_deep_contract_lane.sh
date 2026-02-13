#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/run_live_network_pilot_deep_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/runtime/live_network_pilot_deep_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_live_network_pilot_deep_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_live_network_pilot_deep_lane.sh"

if [[ ! -x "$CONTRACT_LANE" ]]; then
  echo "expected live-network pilot deep contract lane script to be executable" >&2
  exit 1
fi

if [[ ! -x "$DEEP_LANE" ]]; then
  echo "expected live-network pilot deep lane script to be executable" >&2
  exit 1
fi

if [[ ! -x "$SHARED_CONTRACT" ]]; then
  echo "expected live-network pilot deep shared contract module to be executable" >&2
  exit 1
fi

if ! grep -q "run_live_network_pilot_deep_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected live-network pilot deep contract lane to execute deep lane runner" >&2
  exit 1
fi

contract_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$contract_output" | grep -q "live-network pilot deep contract lane tests passed."; then
  echo "expected live-network pilot deep contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$CONTRACT_LANE" ]; then
  echo "expected live-network pilot deep wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$CONTRACT_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected live-network pilot deep wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$CONTRACT_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected live-network pilot deep wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "live_network_pilot_deep_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected live-network pilot deep manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "live-network pilot deep contract lane script tests passed."
