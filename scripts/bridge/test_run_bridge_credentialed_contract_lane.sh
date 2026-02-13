#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credentialed_contract_lane.sh"
FAST_IMPL_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credentialed_contract_lane_impl.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credentialed_deep_lane.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_bridge_credentialed_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected bridge credentialed fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected bridge credentialed deep-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$FAST_IMPL_SCRIPT" ]; then
  echo "expected bridge credentialed contract lane implementation script to be executable" >&2
  exit 1
fi

if [ ! -f "$MANIFEST_FILE" ]; then
  echo "expected bridge credentialed contract lane manifest to exist" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "bridge credentialed contract lane tests passed." "$TMP_OUT"; then
  echo "expected bridge credentialed contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_credentialed_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute fast-lane credential checks first" >&2
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

if ! grep -q "bridge-credential-redaction-report.json" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to emit bridge credential redaction report" >&2
  exit 1
fi

if ! grep -q "bridge-outbound-intent-deep-report.json" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to emit outbound intent deep report" >&2
  exit 1
fi

echo "bridge credentialed contract lane script tests passed."
