#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/signer/run_secure_provider_key_lifecycle_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected secure-provider key-lifecycle contract lane script to be executable" >&2
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

echo "secure-provider key-lifecycle contract lane script tests passed."
