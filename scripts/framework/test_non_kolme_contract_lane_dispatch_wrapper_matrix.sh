#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$DISPATCHER" ]; then
  echo "expected non-Kolme contract-lane dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

lane_wrappers=(
  "run_governance_lifecycle_rollback_contract_lane.sh"
  "run_governance_simulation_contract_lane.sh"
  "run_quorum_attestation_replay_contract_lane.sh"
  "run_stake_slash_risk_contract_lane.sh"
)

for wrapper in "${lane_wrappers[@]}"; do
  wrapper_path="$ROOT_DIR/scripts/governance/$wrapper"

  if [ ! -x "$wrapper_path" ]; then
    echo "expected governance wrapper to be executable: $wrapper_path" >&2
    exit 1
  fi

  if [ ! -L "$wrapper_path" ]; then
    echo "expected governance wrapper to be a symlink to shared dispatcher: $wrapper_path" >&2
    exit 1
  fi

  manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$wrapper" --resolve-manifest-path)"
  if [ ! -f "$manifest_path" ]; then
    echo "expected dispatcher to resolve existing manifest for $wrapper: $manifest_path" >&2
    exit 1
  fi
done

if bash "$DISPATCHER" --lane-wrapper run_missing_non_kolme_contract_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected non-Kolme dispatcher to fail for unknown lane wrapper" >&2
  exit 1
fi

echo "non-Kolme contract lane dispatcher wrapper matrix tests passed."
