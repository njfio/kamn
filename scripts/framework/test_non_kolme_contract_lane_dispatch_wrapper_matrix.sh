#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$DISPATCHER" ]; then
  echo "expected non-Kolme contract-lane dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

lane_wrapper_entries=(
  "scripts/canary/run_launch_canary_contract_lane.sh"
  "scripts/canary/run_post_cutover_slo_contract_lane.sh"
  "scripts/governance/run_governance_lifecycle_rollback_contract_lane.sh"
  "scripts/governance/run_governance_simulation_contract_lane.sh"
  "scripts/governance/run_quorum_attestation_replay_contract_lane.sh"
  "scripts/governance/run_stake_slash_risk_contract_lane.sh"
)

for wrapper_rel in "${lane_wrapper_entries[@]}"; do
  wrapper_path="$ROOT_DIR/$wrapper_rel"
  wrapper_name="$(basename "$wrapper_rel")"

  if [ ! -x "$wrapper_path" ]; then
    echo "expected non-Kolme wrapper to be executable: $wrapper_path" >&2
    exit 1
  fi

  if [ ! -L "$wrapper_path" ]; then
    echo "expected non-Kolme wrapper to be a symlink to shared dispatcher: $wrapper_path" >&2
    exit 1
  fi

  if [ "$(readlink "$wrapper_path")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
    echo "expected non-Kolme wrapper symlink target to be ../framework/run_non_kolme_contract_lane_dispatch.sh: $wrapper_path" >&2
    exit 1
  fi

  manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$wrapper_name" --resolve-manifest-path)"
  if [ ! -f "$manifest_path" ]; then
    echo "expected dispatcher to resolve existing manifest for $wrapper_name: $manifest_path" >&2
    exit 1
  fi
done

if bash "$DISPATCHER" --lane-wrapper run_missing_non_kolme_contract_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected non-Kolme dispatcher to fail for unknown lane wrapper" >&2
  exit 1
fi

echo "non-Kolme contract lane dispatcher wrapper matrix tests passed."
