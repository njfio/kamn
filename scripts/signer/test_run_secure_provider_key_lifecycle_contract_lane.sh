#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/signer/run_secure_provider_key_lifecycle_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/signer/secure_provider_key_lifecycle_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/signer_secure_provider_key_lifecycle_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected secure-provider key-lifecycle contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected secure-provider key-lifecycle shared contract-lane module to be executable" >&2
  exit 1
fi

if ! grep -q "generate_secure_provider_key_lifecycle_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected secure-provider key-lifecycle shared contract module to execute evidence bundle generator" >&2
  exit 1
fi

if ! grep -q "check_secure_provider_key_lifecycle_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected secure-provider key-lifecycle shared contract module to execute policy checker" >&2
  exit 1
fi

bundle_file="$TMP_DIR/secure-provider-key-lifecycle-contract-bundle.json"
output="$(bash "$SCRIPT" --output-file "$bundle_file" --skip-tests)"

if ! printf '%s\n' "$output" | grep -q "secure-provider key-lifecycle contract lane tests passed."; then
  echo "expected success output from secure-provider key-lifecycle contract lane" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected secure-provider key-lifecycle contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.signer.secure-provider-key-lifecycle.v1"' "$bundle_file"; then
  echo "expected secure-provider key-lifecycle evidence schema marker" >&2
  exit 1
fi

if ! grep -q '"reason_key": "secure_provider_key_lifecycle_reason_codes:GO:v1"' "$bundle_file"; then
  echo "expected secure-provider key-lifecycle GO reason key marker in emitted bundle" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected secure-provider key-lifecycle contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected secure-provider key-lifecycle contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected secure-provider key-lifecycle wrapper to resolve signer lifecycle manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "secure_provider_key_lifecycle_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected secure-provider key-lifecycle manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "secure-provider key-lifecycle contract lane script tests passed."
