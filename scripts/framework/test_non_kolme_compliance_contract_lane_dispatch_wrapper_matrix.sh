#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$DISPATCHER" ]; then
  echo "expected non-Kolme contract-lane dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

lane_wrappers=(
  "run_classification_redaction_contract_lane.sh"
  "run_dsar_legal_hold_contract_lane.sh"
  "run_soc2_control_evidence_contract_lane.sh"
)

for wrapper in "${lane_wrappers[@]}"; do
  wrapper_path="$ROOT_DIR/scripts/compliance/$wrapper"

  if [ ! -x "$wrapper_path" ]; then
    echo "expected compliance wrapper to be executable: $wrapper_path" >&2
    exit 1
  fi

  if [ ! -L "$wrapper_path" ]; then
    echo "expected compliance wrapper to be a symlink to shared dispatcher: $wrapper_path" >&2
    exit 1
  fi

  manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$wrapper" --resolve-manifest-path)"
  if [ ! -f "$manifest_path" ]; then
    echo "expected dispatcher to resolve existing manifest for $wrapper: $manifest_path" >&2
    exit 1
  fi
done

if bash "$DISPATCHER" --lane-wrapper run_missing_non_kolme_compliance_contract_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected non-Kolme dispatcher to fail for unknown compliance lane wrapper" >&2
  exit 1
fi

echo "non-Kolme compliance contract lane dispatcher wrapper matrix tests passed."
