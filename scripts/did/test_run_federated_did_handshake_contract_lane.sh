#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/did/run_federated_did_handshake_contract_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/did/run_federated_did_handshake_deep_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected federated DID handshake contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected federated DID handshake deep lane script to be executable" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "federated DID handshake contract lane tests passed."; then
  echo "expected federated DID handshake contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_federated_did_handshake_contract_lane.sh" "$DEEP_LANE"; then
  echo "expected federated DID handshake deep lane script to invoke contract lane baseline checks first" >&2
  exit 1
fi

if ! grep -q "federated-did-handshake-report.json" "$DEEP_LANE"; then
  echo "expected federated DID handshake deep lane script to emit report artifact" >&2
  exit 1
fi

echo "federated DID handshake contract lane script tests passed."
