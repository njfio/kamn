#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/did/run_lifecycle_operator_binding_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected lifecycle operator-binding contract lane script to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/lifecycle-operator-binding-contract-bundle.json"
output="$(bash "$SCRIPT" --output-file "$bundle_file" --skip-tests)"

if ! printf '%s\n' "$output" | grep -q "lifecycle operator-binding contract lane tests passed."; then
  echo "expected success output from lifecycle operator-binding contract lane" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected lifecycle operator-binding contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.did.lifecycle-operator-binding.v1"' "$bundle_file"; then
  echo "expected lifecycle operator-binding evidence schema marker" >&2
  exit 1
fi

if ! grep -q '"reason_key": "did_lifecycle_operator_binding_reason_codes:GO:v1"' "$bundle_file"; then
  echo "expected lifecycle operator-binding reason key marker in emitted bundle" >&2
  exit 1
fi

echo "lifecycle operator-binding contract lane script tests passed."
