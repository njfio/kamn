#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/bridge/run_localhost_bridge_demo_evidence_contract_lane.sh"

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected localhost bridge demo evidence contract lane script to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$LANE_SCRIPT" >"$TMP_OUT"

required_markers=(
  "localhost_bridge_demo_evidence=generated"
  "localhost_bridge_demo_policy=ok"
  "localhost bridge demo evidence contract lane tests passed."
)

for marker in "${required_markers[@]}"; do
  if ! grep -Fq -- "$marker" "$TMP_OUT"; then
    echo "expected localhost bridge demo evidence lane output marker '$marker'" >&2
    exit 1
  fi
done

if ! grep -q "run_localhost_bridge_relay_demo_contract_lane.sh" "$LANE_SCRIPT"; then
  echo "expected localhost bridge demo evidence lane to execute localhost relay contract baseline" >&2
  exit 1
fi

if ! grep -q "run_bridge_replay_matrix.sh" "$LANE_SCRIPT"; then
  echo "expected localhost bridge demo evidence lane to execute replay matrix evidence capture" >&2
  exit 1
fi

if ! grep -q "generate_localhost_bridge_demo_evidence_bundle.sh" "$LANE_SCRIPT"; then
  echo "expected localhost bridge demo evidence lane to generate evidence bundle" >&2
  exit 1
fi

if ! grep -q "check_localhost_bridge_demo_policy.sh" "$LANE_SCRIPT"; then
  echo "expected localhost bridge demo evidence lane to enforce policy checker" >&2
  exit 1
fi

echo "localhost bridge demo evidence contract lane script tests passed."
