#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/message/run_key_hierarchy_invariant_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected key hierarchy invariant contract lane script to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/key-lifecycle-invariant-bundle.json"
lane_output="$(bash "$SCRIPT" --output-file "$bundle_file")"

if ! printf '%s\n' "$lane_output" | grep -q "key hierarchy invariant contract lane tests passed."; then
  echo "expected key hierarchy invariant contract lane success marker" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected key hierarchy invariant contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.key-lifecycle.invariant-evidence.v1"' "$bundle_file"; then
  echo "expected key lifecycle invariant evidence schema marker" >&2
  exit 1
fi

echo "key hierarchy invariant contract lane script tests passed."
