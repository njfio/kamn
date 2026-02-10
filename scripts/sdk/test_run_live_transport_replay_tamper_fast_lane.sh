#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/sdk/run_live_transport_replay_tamper_fast_lane.sh"
SHARED_SCRIPT="$ROOT_DIR/scripts/sdk/live_transport_replay_tamper_contract_lane_contract.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected live transport replay/tamper fast lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_SCRIPT" ]; then
  echo "expected shared replay/tamper lane contract implementation to be executable" >&2
  exit 1
fi

if ! grep -q 'run_live_transport_replay_tamper_contract_lane.sh' "$SCRIPT"; then
  echo "expected replay/tamper fast lane wrapper to delegate to contract lane script" >&2
  exit 1
fi

report_file="$TMP_DIR/live-transport-replay-tamper-fast-report.json"
output="$(bash "$SCRIPT" --output-report "$report_file")"

if ! printf '%s\n' "$output" | grep -q 'lane_mode=fast'; then
  echo "expected replay/tamper fast lane mode marker" >&2
  exit 1
fi

if ! printf '%s\n' "$output" | grep -q 'final_decision=GO'; then
  echo "expected replay/tamper fast lane final decision marker" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.sdk.live-transport-replay-tamper-evidence.v1"' "$report_file"; then
  echo "expected replay/tamper fast lane report schema marker" >&2
  exit 1
fi

echo "live transport replay/tamper fast lane tests passed."
