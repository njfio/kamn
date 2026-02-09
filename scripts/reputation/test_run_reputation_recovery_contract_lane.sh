#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/reputation/run_reputation_recovery_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected reputation recovery contract lane script to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/reputation-recovery-contract-bundle.json"
output="$(bash "$SCRIPT" --output-file "$bundle_file")"

if ! printf '%s\n' "$output" | grep -q "reputation recovery contract lane tests passed."; then
  echo "expected success output from reputation recovery contract lane" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected reputation recovery contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.reputation.recovery-reversal-evidence.v1"' "$bundle_file"; then
  echo "expected reputation recovery evidence schema marker" >&2
  exit 1
fi

if ! grep -q '"reason_key": "reputation_recovery_reason_codes:GO:v1"' "$bundle_file"; then
  echo "expected reputation recovery reason key marker in emitted bundle" >&2
  exit 1
fi

if ! grep -q "check_reputation_recovery_policy.sh" "$SCRIPT"; then
  echo "expected reputation recovery contract lane to execute policy checker" >&2
  exit 1
fi

echo "reputation recovery contract lane script tests passed."
