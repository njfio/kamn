#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/sdk/run_sdk_schema_compatibility_contract_lane.sh"
SHARED_SCRIPT="$ROOT_DIR/scripts/sdk/sdk_schema_compatibility_contract_lane_contract.py"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/sdk_schema_compatibility_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected sdk schema compatibility contract lane script to be executable" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected sdk schema compatibility contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected sdk schema compatibility contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected sdk schema compatibility wrapper to resolve sdk manifest via dispatcher" >&2
  exit 1
fi

if [ ! -x "$SHARED_SCRIPT" ]; then
  echo "expected shared sdk schema compatibility contract lane implementation to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/sdk-schema-compatibility-contract-bundle.json"
output="$(bash "$SCRIPT" --output-file "$bundle_file")"

if ! printf '%s\n' "$output" | grep -q "sdk schema compatibility contract lane tests passed."; then
  echo "expected success output from sdk schema compatibility contract lane" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected sdk schema compatibility contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.sdk.schema-compatibility-evidence.v1"' "$bundle_file"; then
  echo "expected sdk schema compatibility evidence schema marker" >&2
  exit 1
fi

if ! grep -q '"reason_key": "sdk_schema_compatibility_reason_codes:GO:v1"' "$bundle_file"; then
  echo "expected sdk schema compatibility reason key marker in emitted bundle" >&2
  exit 1
fi

if ! grep -q "check_sdk_schema_compatibility_policy.sh" "$SHARED_SCRIPT"; then
  echo "expected shared sdk schema compatibility contract lane implementation to execute policy checker" >&2
  exit 1
fi

if ! grep -q "sdk_schema_compatibility_contract_lane_contract.py" "$MANIFEST_FILE"; then
  echo "expected sdk schema compatibility manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "sdk schema compatibility contract lane script tests passed."
