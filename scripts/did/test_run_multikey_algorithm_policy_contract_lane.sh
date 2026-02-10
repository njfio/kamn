#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/did/run_multikey_algorithm_policy_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected multikey algorithm policy contract lane script to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/did-multikey-algorithm-policy-contract-bundle.json"
lane_output="$(bash "$SCRIPT" --skip-tests --output-file "$bundle_file")"

if ! printf '%s\n' "$lane_output" | grep -q "multikey algorithm policy contract lane tests passed."; then
  echo "expected multikey algorithm policy contract lane success marker" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected multikey algorithm policy contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.did.multikey-algorithm-policy-report.v1"' "$bundle_file"; then
  echo "expected multikey algorithm policy evidence schema marker" >&2
  exit 1
fi

echo "multikey algorithm policy contract lane script tests passed."
