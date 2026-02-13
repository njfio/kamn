#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_contract_lane.sh"
CONTRACT_LANE_IMPL="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_contract_lane_impl.sh"
DEEP_LANE="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_deep_lane.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_cross_chain_outbound_intent_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected outbound intent contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected outbound intent deep lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$CONTRACT_LANE_IMPL" ]; then
  echo "expected outbound intent contract lane implementation script to be executable" >&2
  exit 1
fi

if [ ! -f "$MANIFEST_FILE" ]; then
  echo "expected outbound intent contract lane manifest to exist" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "cross-chain outbound intent contract lane tests passed."; then
  echo "expected outbound intent contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$CONTRACT_LANE" ]; then
  echo "expected outbound intent contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$CONTRACT_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected outbound intent contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$CONTRACT_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected outbound intent wrapper to resolve bridge manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_cross_chain_outbound_intent_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected outbound intent manifest to dispatch to implementation script" >&2
  exit 1
fi

if ! grep -Fq "run_cross_chain_outbound_intent_contract_lane.sh" "$DEEP_LANE"; then
  echo "expected deep lane script to invoke outbound intent contract lane baseline checks first" >&2
  exit 1
fi

if ! grep -q "bridge-outbound-intent-deep-report.json" "$DEEP_LANE"; then
  echo "expected deep lane script to emit outbound intent deep report artifact" >&2
  exit 1
fi

echo "cross-chain outbound intent contract lane script tests passed."
