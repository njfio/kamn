#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
FAST_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_ingress_relay_contract_lane.sh"
FAST_IMPL_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_ingress_relay_contract_lane_impl.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_bridge_ingress_relay_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

test_harness_require_executable "$FAST_SCRIPT" "expected bridge ingress relay contract lane script to be executable"

test_harness_require_executable "$FAST_IMPL_SCRIPT" "expected bridge ingress relay contract lane implementation script to be executable"

test_harness_require_file "$MANIFEST_FILE" "expected bridge ingress relay contract lane manifest to exist"

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" --skip-replay >"$TMP_OUT"
if ! grep -q "bridge ingress relay contract lane tests passed." "$TMP_OUT"; then
  echo "expected bridge ingress relay contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected bridge ingress relay contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected bridge ingress relay contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected bridge ingress relay wrapper to resolve bridge manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_ingress_relay_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected bridge ingress relay manifest to dispatch to implementation script" >&2
  exit 1
fi

if ! grep -q "bridge_ingress_relay_harness" "$FAST_IMPL_SCRIPT"; then
  echo "expected ingress relay implementation lane to execute bridge ingress relay harness tests" >&2
  exit 1
fi

if ! grep -q "bridge_adapter,telegram_bridge,discord_bridge" "$FAST_IMPL_SCRIPT"; then
  echo "expected ingress relay implementation lane replay selection to cover bridge adapter + telegram + discord suites" >&2
  exit 1
fi

echo "bridge ingress relay contract lane script tests passed."
