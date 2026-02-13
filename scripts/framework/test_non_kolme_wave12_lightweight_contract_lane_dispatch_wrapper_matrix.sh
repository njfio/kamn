#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$DISPATCHER" ]; then
  echo "expected non-Kolme contract-lane dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

lane_wrappers=(
  "scripts/signer/run_secure_provider_key_lifecycle_contract_lane.sh"
  "scripts/cutover/run_mainnet_cutover_contract_lane.sh"
  "scripts/deploy/run_gonogo_evidence_contract_lane.sh"
  "scripts/deploy/run_staging_rehearsal_contract_lane.sh"
)

for wrapper_rel_path in "${lane_wrappers[@]}"; do
  wrapper_path="$ROOT_DIR/$wrapper_rel_path"
  wrapper_name="$(basename "$wrapper_path")"

  if [ ! -x "$wrapper_path" ]; then
    echo "expected lightweight wrapper to be executable: $wrapper_path" >&2
    exit 1
  fi

  if [ ! -L "$wrapper_path" ]; then
    echo "expected lightweight wrapper to be a symlink to shared dispatcher: $wrapper_path" >&2
    exit 1
  fi

  manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$wrapper_name" --resolve-manifest-path)"
  if [ ! -f "$manifest_path" ]; then
    echo "expected dispatcher to resolve existing manifest for $wrapper_name: $manifest_path" >&2
    exit 1
  fi
done

if bash "$DISPATCHER" --lane-wrapper run_missing_non_kolme_wave12_lightweight_contract_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected non-Kolme dispatcher to fail for unknown wave-12 lightweight wrapper" >&2
  exit 1
fi

echo "non-Kolme wave-12 lightweight contract lane dispatcher wrapper matrix tests passed."
