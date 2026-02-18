#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
FAST_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credentialed_contract_lane.sh"
FAST_IMPL_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credentialed_contract_lane_impl.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credentialed_deep_lane.sh"
DEEP_IMPL_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credentialed_deep_lane_impl.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_bridge_credentialed_contract_lane.json"
DEEP_MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_bridge_credentialed_deep_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

test_harness_require_executable "$FAST_SCRIPT" "expected bridge credentialed fast-lane runner to be executable"

test_harness_require_executable "$DEEP_SCRIPT" "expected bridge credentialed deep-lane runner to be executable"

test_harness_require_executable "$DEEP_IMPL_SCRIPT" "expected bridge credentialed deep-lane implementation script to be executable"

test_harness_require_executable "$FAST_IMPL_SCRIPT" "expected bridge credentialed contract lane implementation script to be executable"

test_harness_require_file "$MANIFEST_FILE" "expected bridge credentialed contract lane manifest to exist"

test_harness_require_file "$DEEP_MANIFEST_FILE" "expected bridge credentialed deep-lane manifest to exist"

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "bridge credentialed contract lane tests passed." "$TMP_OUT"; then
  echo "expected bridge credentialed contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$DEEP_SCRIPT" ]; then
  echo "expected bridge credentialed deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$DEEP_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected bridge credentialed deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_deep_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_deep_manifest" != "$DEEP_MANIFEST_FILE" ]; then
  echo "expected bridge credentialed deep wrapper to resolve bridge deep manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_credentialed_deep_lane_impl.sh" "$DEEP_MANIFEST_FILE"; then
  echo "expected bridge credentialed deep manifest to dispatch to implementation script" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_credentialed_contract_lane.sh" "$DEEP_IMPL_SCRIPT"; then
  echo "expected deep-lane implementation script to execute fast-lane credential checks first" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected bridge credentialed contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected bridge credentialed contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected bridge credentialed wrapper to resolve bridge manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_credentialed_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected bridge credentialed manifest to dispatch to implementation script" >&2
  exit 1
fi

if ! grep -q "run_cross_chain_outbound_intent_contract_lane.sh" "$FAST_IMPL_SCRIPT"; then
  echo "expected bridge credentialed implementation script to execute outbound intent contract lane" >&2
  exit 1
fi

if ! grep -q "bridge-credential-redaction-report.json" "$DEEP_IMPL_SCRIPT"; then
  echo "expected deep-lane implementation script to emit bridge credential redaction report" >&2
  exit 1
fi

if ! grep -q "bridge-outbound-intent-deep-report.json" "$DEEP_IMPL_SCRIPT"; then
  echo "expected deep-lane implementation script to emit outbound intent deep report" >&2
  exit 1
fi

echo "bridge credentialed contract lane script tests passed."
