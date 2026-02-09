#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/channel/run_channel_retention_redaction_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected channel retention/redaction contract lane script to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/channel-retention-redaction-bundle.json"
lane_output="$(bash "$SCRIPT" --output-file "$bundle_file")"

if ! printf '%s\n' "$lane_output" | grep -q "channel retention/redaction contract lane tests passed."; then
  echo "expected channel retention/redaction contract lane success marker" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected channel retention/redaction contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.channel.retention-redaction-evidence.v1"' "$bundle_file"; then
  echo "expected channel retention/redaction evidence schema marker" >&2
  exit 1
fi

echo "channel retention/redaction contract lane script tests passed."
