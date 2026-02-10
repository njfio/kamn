#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/sdk/run_live_transport_replay_tamper_deep_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected live transport replay/tamper deep lane script to be executable" >&2
  exit 1
fi

if ! grep -q 'run_live_transport_replay_tamper_contract_lane.sh' "$SCRIPT"; then
  echo "expected replay/tamper deep lane wrapper to delegate to contract lane script" >&2
  exit 1
fi

report_file="$TMP_DIR/live-transport-replay-tamper-deep-report.json"
output="$(bash "$SCRIPT" --output-report "$report_file")"

if ! printf '%s\n' "$output" | grep -q 'lane_mode=deep'; then
  echo "expected replay/tamper deep lane mode marker" >&2
  exit 1
fi

if ! printf '%s\n' "$output" | grep -q 'deep_no_go_status=verified'; then
  echo "expected replay/tamper deep lane NO-GO verification marker" >&2
  exit 1
fi

if ! printf '%s\n' "$output" | grep -q 'final_decision=GO'; then
  echo "expected replay/tamper deep lane final decision marker" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.sdk.live-transport-replay-tamper-evidence.v1"' "$report_file"; then
  echo "expected replay/tamper deep lane report schema marker" >&2
  exit 1
fi

echo "live transport replay/tamper deep lane tests passed."
