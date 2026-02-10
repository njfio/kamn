#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/did/run_service_endpoint_canonicalization_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected service endpoint canonicalization contract lane script to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/did-service-endpoint-canonicalization-contract-bundle.json"
lane_output="$(bash "$SCRIPT" --skip-tests --output-file "$bundle_file")"

if ! printf '%s\n' "$lane_output" | grep -q "service endpoint canonicalization contract lane tests passed."; then
  echo "expected service endpoint canonicalization contract lane success marker" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected service endpoint canonicalization contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.did.service-endpoint-canonicalization-report.v1"' "$bundle_file"; then
  echo "expected service endpoint canonicalization evidence schema marker" >&2
  exit 1
fi

echo "service endpoint canonicalization contract lane script tests passed."
