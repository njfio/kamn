#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/runtime/run_processor_proof_admission_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected processor proof admission contract lane script to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/processor-proof-admission-contract-bundle.json"
lane_output="$(bash "$SCRIPT" --output-file "$bundle_file")"

if ! printf '%s\n' "$lane_output" | grep -q "processor proof admission contract lane tests passed."; then
  echo "expected processor proof admission contract lane success marker" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected processor proof admission contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.runtime.processor-proof-admission-report.v1"' "$bundle_file"; then
  echo "expected processor proof admission evidence schema marker" >&2
  exit 1
fi

echo "processor proof admission contract lane script tests passed."
