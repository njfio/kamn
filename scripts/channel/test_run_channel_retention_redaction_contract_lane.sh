#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
SCRIPT="$ROOT_DIR/scripts/channel/run_channel_retention_redaction_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
CONTRACT_MODULE="$ROOT_DIR/scripts/channel/channel_retention_redaction_contract_lane_contract.sh"
EXPECTED_MANIFEST="$ROOT_DIR/scripts/framework/manifests/channel_channel_retention_redaction_contract_lane.json"

test_harness_require_executable "$SCRIPT" "expected channel retention/redaction contract lane script to be executable"

test_harness_require_executable "$DISPATCHER" "expected non-Kolme dispatcher to be executable: $DISPATCHER"

test_harness_require_executable "$CONTRACT_MODULE" "expected shared channel retention/redaction contract module to be executable: $CONTRACT_MODULE"

if [ ! -L "$SCRIPT" ]; then
  echo "expected channel retention/redaction wrapper to be a symlink: $SCRIPT" >&2
  exit 1
fi

manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$manifest_path" != "$EXPECTED_MANIFEST" ]; then
  echo "expected dispatcher to resolve $EXPECTED_MANIFEST but found $manifest_path" >&2
  exit 1
fi

if ! grep -Fq "\"scripts/channel/$(basename "$CONTRACT_MODULE")\"" "$manifest_path"; then
  echo "expected manifest to dispatch shared channel retention/redaction contract module: $manifest_path" >&2
  exit 1
fi

echo "channel retention/redaction contract lane wrapper tests passed."
