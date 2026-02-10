#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/message/run_processor_proof_artifact_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected processor proof artifact contract lane script to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/processor-proof-artifact-contract-bundle.json"
lane_output="$(bash "$SCRIPT" --output-file "$bundle_file")"

if ! printf '%s\n' "$lane_output" | grep -q "processor proof artifact contract lane tests passed."; then
  echo "expected processor proof artifact contract lane success marker" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected processor proof artifact contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.zk.processor-proof-artifact-evidence.v1"' "$bundle_file"; then
  echo "expected processor proof artifact evidence schema marker" >&2
  exit 1
fi

echo "processor proof artifact contract lane script tests passed."
