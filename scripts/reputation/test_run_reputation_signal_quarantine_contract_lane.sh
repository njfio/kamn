#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/reputation/run_reputation_signal_quarantine_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected reputation signal quarantine contract lane script to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/reputation-signal-quarantine-contract-bundle.json"
output="$(bash "$SCRIPT" --output-file "$bundle_file")"

if ! printf '%s\n' "$output" | grep -q "reputation signal quarantine contract lane tests passed."; then
  echo "expected success output from signal quarantine contract lane" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected signal quarantine contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.reputation.signal-quarantine-evidence.v1"' "$bundle_file"; then
  echo "expected signal quarantine evidence schema marker" >&2
  exit 1
fi

if ! grep -q '"reason_key": "reputation_signal_quarantine_reason_codes:GO:v1"' "$bundle_file"; then
  echo "expected signal quarantine reason key marker in emitted bundle" >&2
  exit 1
fi

if ! grep -q "check_reputation_signal_quarantine_policy.sh" "$SCRIPT"; then
  echo "expected signal quarantine contract lane to execute policy checker" >&2
  exit 1
fi

echo "reputation signal quarantine contract lane script tests passed."
