#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_redaction_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_redaction_deep_lane.sh"
REPLAY_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_matrix.sh"
REPLAY_FIXTURE="$ROOT_DIR/fixtures/bridge_replay/replay_validation_cases.json"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected bridge replay/redaction contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected bridge replay/redaction deep lane script to be executable" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

replay_report="$TMP_DIR/replay-report.json"
bash "$REPLAY_SCRIPT" \
  --fixture "$REPLAY_FIXTURE" \
  --suites "bridge_adapter,discord_bridge" \
  --output-json "$replay_report" >/dev/null

bundle_file="$TMP_DIR/bridge-replay-redaction-contract-bundle.json"
lane_output="$(bash "$FAST_SCRIPT" --skip-replay --replay-report-file "$replay_report" --output-bundle "$bundle_file")"
if ! printf '%s\n' "$lane_output" | grep -q "bridge replay redaction contract lane tests passed."; then
  echo "expected bridge replay/redaction contract lane success marker" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected bridge replay/redaction contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.bridge.replay-redaction-evidence.v1"' "$bundle_file"; then
  echo "expected bridge replay/redaction evidence bundle schema version marker" >&2
  exit 1
fi

if ! grep -q '"final_decision": "GO"' "$bundle_file"; then
  echo "expected bridge replay/redaction contract bundle GO final decision" >&2
  exit 1
fi

if ! grep -q -- "--skip-replay" "$FAST_SCRIPT"; then
  echo "expected bridge replay/redaction contract lane to support skip-replay mode" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_replay_redaction_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected bridge replay/redaction deep lane to execute contract lane baseline checks first" >&2
  exit 1
fi

if ! grep -q "bridge-replay-redaction-deep-bundle.json" "$DEEP_SCRIPT"; then
  echo "expected bridge replay/redaction deep lane to emit deep evidence bundle artifact" >&2
  exit 1
fi

echo "bridge replay/redaction contract lane script tests passed."
