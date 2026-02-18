#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_contract_lane.sh"
CONTRACT_LANE_IMPL="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_contract_lane_impl.sh"
DEEP_LANE="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_deep_lane.sh"
DEEP_LANE_IMPL="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_deep_lane_impl.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_cross_chain_outbound_intent_contract_lane.json"
DEEP_MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_cross_chain_outbound_intent_deep_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

test_harness_require_executable "$CONTRACT_LANE" "expected outbound intent contract lane script to be executable"

test_harness_require_executable "$DEEP_LANE" "expected outbound intent deep lane script to be executable"

test_harness_require_executable "$DEEP_LANE_IMPL" "expected outbound intent deep lane implementation script to be executable"

test_harness_require_executable "$CONTRACT_LANE_IMPL" "expected outbound intent contract lane implementation script to be executable"

test_harness_require_file "$MANIFEST_FILE" "expected outbound intent contract lane manifest to exist"

test_harness_require_file "$DEEP_MANIFEST_FILE" "expected outbound intent deep lane manifest to exist"

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

if [ ! -L "$DEEP_LANE" ]; then
  echo "expected outbound intent deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$DEEP_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected outbound intent deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_deep_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_LANE")" --resolve-manifest-path)"
if [ "$resolved_deep_manifest" != "$DEEP_MANIFEST_FILE" ]; then
  echo "expected outbound intent deep wrapper to resolve bridge deep manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_cross_chain_outbound_intent_deep_lane_impl.sh" "$DEEP_MANIFEST_FILE"; then
  echo "expected outbound intent deep manifest to dispatch to implementation script" >&2
  exit 1
fi

if ! grep -Fq "run_cross_chain_outbound_intent_contract_lane.sh" "$DEEP_LANE_IMPL"; then
  echo "expected deep lane implementation script to invoke outbound intent contract lane baseline checks first" >&2
  exit 1
fi

if ! grep -q "bridge-outbound-intent-deep-report.json" "$DEEP_LANE_IMPL"; then
  echo "expected deep lane implementation script to emit outbound intent deep report artifact" >&2
  exit 1
fi

echo "cross-chain outbound intent contract lane script tests passed."
