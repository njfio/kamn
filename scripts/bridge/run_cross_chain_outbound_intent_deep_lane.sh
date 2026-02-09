#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_contract_lane.sh"
MATRIX_SCRIPT="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/bridge_outbound_intent/approval_retry_cases.json"
REPLAY_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_matrix.sh"
REPLAY_FIXTURE="$ROOT_DIR/fixtures/bridge_replay/replay_validation_cases.json"

output_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$output_json" ]]; then
  output_json="$ROOT_DIR/bridge-outbound-intent-deep-report.json"
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

bash "$CONTRACT_LANE" >/dev/null

matrix_output="$(
  python3 "$MATRIX_SCRIPT" \
    --fixture "$FIXTURE_FILE" \
    --output-json "$output_json"
)"
if ! printf '%s\n' "$matrix_output" | grep -q '^status=pass;'; then
  echo "expected outbound intent deep matrix to pass" >&2
  exit 1
fi

bash "$REPLAY_SCRIPT" \
  --fixture "$REPLAY_FIXTURE" \
  --suites "discord_bridge,cross_chain_bridge" \
  --output-json "$TMP_DIR/bridge-outbound-intent-replay-report.json" >/dev/null

echo "cross-chain outbound intent deep lane tests passed."
