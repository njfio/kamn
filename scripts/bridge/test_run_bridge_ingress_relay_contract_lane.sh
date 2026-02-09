#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_ingress_relay_contract_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected bridge ingress relay contract lane script to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" --skip-replay >"$TMP_OUT"
if ! grep -q "bridge ingress relay contract lane tests passed." "$TMP_OUT"; then
  echo "expected bridge ingress relay contract lane success marker" >&2
  exit 1
fi

if ! grep -q "bridge_ingress_relay_harness" "$FAST_SCRIPT"; then
  echo "expected ingress relay contract lane script to execute bridge ingress relay harness tests" >&2
  exit 1
fi

if ! grep -q "bridge_adapter,telegram_bridge,discord_bridge" "$FAST_SCRIPT"; then
  echo "expected ingress relay contract lane replay selection to cover bridge adapter + telegram + discord suites" >&2
  exit 1
fi

echo "bridge ingress relay contract lane script tests passed."
