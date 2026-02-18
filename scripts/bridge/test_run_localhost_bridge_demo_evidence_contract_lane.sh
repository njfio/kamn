#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
LANE_SCRIPT="$ROOT_DIR/scripts/bridge/run_localhost_bridge_demo_evidence_contract_lane.sh"
LANE_IMPL_SCRIPT="$ROOT_DIR/scripts/bridge/run_localhost_bridge_demo_evidence_contract_lane_impl.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_localhost_bridge_demo_evidence_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

test_harness_require_executable "$LANE_SCRIPT" "expected localhost bridge demo evidence contract lane script to be executable"

test_harness_require_executable "$LANE_IMPL_SCRIPT" "expected localhost bridge demo evidence contract lane implementation script to be executable"

test_harness_require_file "$MANIFEST_FILE" "expected localhost bridge demo evidence contract lane manifest to exist"

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

if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected localhost bridge demo evidence contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected localhost bridge demo evidence contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected localhost bridge demo evidence wrapper to resolve bridge manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_localhost_bridge_demo_evidence_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected localhost bridge demo evidence manifest to dispatch to implementation script" >&2
  exit 1
fi

if ! grep -q "run_localhost_bridge_relay_demo_contract_lane.sh" "$LANE_IMPL_SCRIPT"; then
  echo "expected localhost bridge demo evidence implementation lane to execute localhost relay contract baseline" >&2
  exit 1
fi

if ! grep -q "run_bridge_replay_matrix.sh" "$LANE_IMPL_SCRIPT"; then
  echo "expected localhost bridge demo evidence implementation lane to execute replay matrix evidence capture" >&2
  exit 1
fi

if ! grep -q "generate_localhost_bridge_demo_evidence_bundle.sh" "$LANE_IMPL_SCRIPT"; then
  echo "expected localhost bridge demo evidence implementation lane to generate evidence bundle" >&2
  exit 1
fi

if ! grep -q "check_localhost_bridge_demo_policy.sh" "$LANE_IMPL_SCRIPT"; then
  echo "expected localhost bridge demo evidence implementation lane to enforce policy checker" >&2
  exit 1
fi

echo "localhost bridge demo evidence contract lane script tests passed."
