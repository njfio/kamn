#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/bridge/run_localhost_bridge_relay_demo_contract_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected localhost bridge relay demo contract lane script to be executable" >&2
  exit 1
fi

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

if ! grep -q "run_localhost_signed_demo.sh" "$FAST_SCRIPT"; then
  echo "expected localhost bridge relay demo lane to execute sdk localhost signed demo baseline" >&2
  exit 1
fi

if ! grep -q "bridge_ingress_relay_harness" "$FAST_SCRIPT"; then
  echo "expected localhost bridge relay demo lane to execute ingress relay contracts" >&2
  exit 1
fi

if ! grep -q "bridge_outbound_quorum_execution" "$FAST_SCRIPT"; then
  echo "expected localhost bridge relay demo lane to execute outbound quorum contracts" >&2
  exit 1
fi

echo "localhost bridge relay demo contract lane script tests passed."
