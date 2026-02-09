#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOCALHOST_DEMO_SCRIPT="$ROOT_DIR/scripts/sdk/run_localhost_signed_demo.sh"

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

KAMN_LOCALHOST_SIGNED_DEMO_ADDR="${KAMN_LOCALHOST_SIGNED_DEMO_ADDR:-127.0.0.1:17880}" \
KAMN_LOCALHOST_SIGNED_DEMO_FROM="${KAMN_LOCALHOST_SIGNED_DEMO_FROM:-kamn:did:agent:bridge-telegram-local}" \
KAMN_LOCALHOST_SIGNED_DEMO_TO="${KAMN_LOCALHOST_SIGNED_DEMO_TO:-kamn:did:agent:bridge-discord-local}" \
KAMN_LOCALHOST_SIGNED_DEMO_STATE_HASH="${KAMN_LOCALHOST_SIGNED_DEMO_STATE_HASH:-state:bridge-localhost-relay}" \
KAMN_LOCALHOST_SIGNED_DEMO_BODY="${KAMN_LOCALHOST_SIGNED_DEMO_BODY:-bridge-localhost-demo-message}" \
  bash "$LOCALHOST_DEMO_SCRIPT" >"$TMP_DIR/localhost-demo.out"

if ! grep -q "localhost signed message demo completed." "$TMP_DIR/localhost-demo.out"; then
  echo "expected localhost signed demo completion marker in bridge relay contract lane" >&2
  exit 1
fi

cargo test -p kamn-core --test bridge_ingress_relay_harness -- functional_ingress_fixture_matrix_projects_deterministic_envelopes --exact >/dev/null
cargo test -p kamn-core --test bridge_outbound_quorum_execution -- functional_outbound_quorum_matrix_dispatches_deterministically --exact >/dev/null

echo "bridge_demo_signed_transport=pass"
echo "bridge_demo_relay_contracts=pass"

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 120 ]; then
  echo "localhost bridge relay demo contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "localhost bridge relay demo contract lane tests passed."
