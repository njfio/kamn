#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
FAST_SCRIPT="$ROOT_DIR/scripts/bridge/run_localhost_bridge_relay_demo_contract_lane.sh"
FAST_IMPL_SCRIPT="$ROOT_DIR/scripts/bridge/run_localhost_bridge_relay_demo_contract_lane_impl.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_localhost_bridge_relay_demo_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

test_harness_require_executable "$FAST_SCRIPT" "expected localhost bridge relay demo contract lane script to be executable"

test_harness_require_executable "$FAST_IMPL_SCRIPT" "expected localhost bridge relay demo contract lane implementation script to be executable"

test_harness_require_file "$MANIFEST_FILE" "expected localhost bridge relay demo contract lane manifest to exist"

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"

required_markers=(
  "bridge_demo_signed_transport=pass"
  "bridge_demo_relay_contracts=pass"
  "localhost bridge relay demo contract lane tests passed."
)

for marker in "${required_markers[@]}"; do
  if ! grep -Fq -- "$marker" "$TMP_OUT"; then
    echo "expected localhost bridge relay demo contract lane output marker '$marker'" >&2
    exit 1
  fi
done

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected localhost bridge relay demo contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected localhost bridge relay demo contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected localhost bridge relay demo wrapper to resolve bridge manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_localhost_bridge_relay_demo_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected localhost bridge relay demo manifest to dispatch to implementation script" >&2
  exit 1
fi

if ! grep -q "run_localhost_signed_demo.sh" "$FAST_IMPL_SCRIPT"; then
  echo "expected localhost bridge relay demo implementation lane to execute sdk localhost signed demo baseline" >&2
  exit 1
fi

if ! grep -q "bridge_ingress_relay_harness" "$FAST_IMPL_SCRIPT"; then
  echo "expected localhost bridge relay demo implementation lane to execute ingress relay contracts" >&2
  exit 1
fi

if ! grep -q "bridge_outbound_quorum_execution" "$FAST_IMPL_SCRIPT"; then
  echo "expected localhost bridge relay demo implementation lane to execute outbound quorum contracts" >&2
  exit 1
fi

echo "localhost bridge relay demo contract lane script tests passed."
