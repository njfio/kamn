#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/sdk/run_live_transport_smoke_parity_contract_lane.sh"
SHARED_SCRIPT="$ROOT_DIR/scripts/sdk/live_transport_smoke_parity_contract_lane_contract.py"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/sdk_live_transport_smoke_parity_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected sdk smoke parity contract lane script to be executable" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected sdk smoke parity contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected sdk smoke parity contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected sdk smoke parity wrapper to resolve sdk manifest via dispatcher" >&2
  exit 1
fi

if [ ! -x "$SHARED_SCRIPT" ]; then
  echo "expected shared sdk smoke parity contract lane implementation to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/sdk-live-transport-smoke-parity-contract-report.json"
output="$(
  KAMN_SDK_SMOKE_PARITY_CONTRACT_MAX_SECONDS=300 \
  bash "$SCRIPT" --output-file "$report_file"
)"

if ! printf '%s\n' "$output" | grep -q 'sdk live transport smoke parity contract lane tests passed.'; then
  echo "expected success output from sdk smoke parity contract lane" >&2
  exit 1
fi

if [ ! -f "$report_file" ]; then
  echo "expected sdk smoke parity contract lane to emit report file" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.sdk.live-transport-smoke-parity-report.v1"' "$report_file"; then
  echo "expected sdk smoke parity report schema marker in contract lane output" >&2
  exit 1
fi

if ! grep -q '"final_decision": "GO"' "$report_file"; then
  echo "expected GO final decision in sdk smoke parity contract lane report" >&2
  exit 1
fi

if ! grep -q '"skip_commands": true' "$report_file"; then
  echo "expected sdk smoke parity contract lane GO path to default to deterministic skip_commands=true" >&2
  exit 1
fi

if ! grep -q 'check_live_transport_smoke_parity_policy.sh' "$SHARED_SCRIPT"; then
  echo "expected shared sdk smoke parity contract lane implementation to execute policy checker" >&2
  exit 1
fi

if ! grep -q 'live_transport_smoke_parity_contract_lane_contract.py' "$MANIFEST_FILE"; then
  echo "expected sdk smoke parity manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "sdk live transport smoke parity contract lane script tests passed."
