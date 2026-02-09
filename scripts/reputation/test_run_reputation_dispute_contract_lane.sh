#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/reputation/run_reputation_dispute_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected reputation dispute contract lane script to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/reputation-dispute-contract-bundle.json"
output="$(bash "$SCRIPT" --output-file "$bundle_file")"

if ! printf '%s\n' "$output" | grep -q "reputation dispute contract lane tests passed."; then
  echo "expected success output from reputation dispute contract lane" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected reputation dispute contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.reputation.dispute-evidence.v1"' "$bundle_file"; then
  echo "expected reputation dispute evidence schema marker" >&2
  exit 1
fi

if ! grep -q '"reason_key": "reputation_dispute_reason_codes:GO:v1"' "$bundle_file"; then
  echo "expected reputation dispute reason key marker in emitted bundle" >&2
  exit 1
fi

echo "reputation dispute contract lane script tests passed."
