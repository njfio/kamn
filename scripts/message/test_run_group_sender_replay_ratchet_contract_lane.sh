#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/message/run_group_sender_replay_ratchet_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected group sender replay/ratchet contract lane script to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/group-sender-replay-ratchet-bundle.json"
lane_output="$(bash "$SCRIPT" --output-file "$bundle_file")"

if ! printf '%s\n' "$lane_output" | grep -q "group sender replay/ratchet contract lane tests passed."; then
  echo "expected group sender replay/ratchet contract lane success marker" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected group sender replay/ratchet contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.group-sender.replay-ratchet-evidence.v1"' "$bundle_file"; then
  echo "expected group sender replay/ratchet evidence schema marker" >&2
  exit 1
fi

echo "group sender replay/ratchet contract lane script tests passed."
