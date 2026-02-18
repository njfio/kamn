#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
SCRIPT="$ROOT_DIR/scripts/channel/run_channel_policy_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/channel/channel_policy_contract_lane_contract.py"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/channel_channel_policy_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

test_harness_require_executable "$SCRIPT" "expected channel policy contract lane script to be executable"
test_harness_require_executable "$SHARED_CONTRACT" "expected channel policy shared contract-lane module to be executable"

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$SCRIPT" >"$TMP_OUT"

if ! grep -q "channel policy contract lane tests passed." "$TMP_OUT"; then
  echo "expected channel policy contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected channel policy contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected channel policy contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected channel policy wrapper to resolve channel manifest via dispatcher" >&2
  exit 1
fi
if ! grep -Fq "channel_policy_contract_lane_contract.py" "$MANIFEST_FILE"; then
  echo "expected channel policy manifest to dispatch to shared contract module" >&2
  exit 1
fi
if ! grep -Fq "run_channel_retention_redaction_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected channel policy shared contract-lane module to execute retention/redaction contract lane checks" >&2
  exit 1
fi

echo "channel policy contract lane script tests passed."
