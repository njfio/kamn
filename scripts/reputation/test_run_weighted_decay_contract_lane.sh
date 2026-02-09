#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/reputation/run_weighted_decay_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected weighted decay contract lane script to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/weighted-decay-property-contract-bundle.json"
output="$(bash "$SCRIPT" --output-file "$bundle_file")"

if ! printf '%s\n' "$output" | grep -q "weighted decay contract lane tests passed."; then
  echo "expected success output from weighted decay contract lane" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected weighted decay contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.reputation.weighted-decay.property-evidence.v1"' "$bundle_file"; then
  echo "expected weighted decay property evidence schema marker" >&2
  exit 1
fi

if ! grep -q "check_weighted_decay_property_policy.sh" "$SCRIPT"; then
  echo "expected weighted decay contract lane to execute weighted decay policy checker" >&2
  exit 1
fi

echo "weighted decay contract lane script tests passed."
