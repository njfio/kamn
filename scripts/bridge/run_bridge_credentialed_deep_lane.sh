#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPLAY_FIXTURE="$ROOT_DIR/fixtures/bridge_replay/replay_validation_cases.json"
REPLAY_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_matrix.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/bridge/run_bridge_credentialed_contract_lane.sh"
REDACTION_CHECK_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credential_redaction_check.py"
OUTBOUND_INTENT_DEEP_SCRIPT="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_deep_lane.sh"

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

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BRIDGE_REPLAY_DEEP_REPORT="$TMP_DIR/bridge-replay-deep-report.json"
BRIDGE_CREDENTIAL_REDACTION_DEEP_REPORT="$TMP_DIR/bridge-credential-redaction-report.json"
BRIDGE_DEEP_STDOUT="$TMP_DIR/bridge-redaction-deep.out"
BRIDGE_OUTBOUND_INTENT_DEEP_REPORT="$TMP_DIR/bridge-outbound-intent-deep-report.json"

bash "$CONTRACT_LANE"
bash "$OUTBOUND_INTENT_DEEP_SCRIPT" --output-json "$BRIDGE_OUTBOUND_INTENT_DEEP_REPORT" >/dev/null

bash "$REPLAY_SCRIPT" \
  --fixture "$REPLAY_FIXTURE" \
  --suites "bridge_adapter,telegram_bridge,discord_bridge,cross_chain_bridge" \
  --output-json "$BRIDGE_REPLAY_DEEP_REPORT" >/dev/null

TELEGRAM_TOKEN="telegram_deep_token_t7y8u9i0o1p2"
DISCORD_TOKEN="discord_deep_token_a1s2d3f4g5h6"
CROSS_CHAIN_TOKEN="crosschain_deep_token_q1w2e3r4t5y6"

python3 "$REDACTION_CHECK_SCRIPT" \
  --mode deep \
  --output-json "$BRIDGE_CREDENTIAL_REDACTION_DEEP_REPORT" \
  --telegram-token "$TELEGRAM_TOKEN" \
  --discord-token "$DISCORD_TOKEN" \
  --cross-chain-token "$CROSS_CHAIN_TOKEN" >"$BRIDGE_DEEP_STDOUT"

if ! grep -q '^status=pass$' "$BRIDGE_DEEP_STDOUT"; then
  echo "expected deep-lane redaction checker to pass" >&2
  exit 1
fi

if ! grep -q '"sample_count": 128' "$BRIDGE_CREDENTIAL_REDACTION_DEEP_REPORT"; then
  echo "expected deep-lane redaction report to include sample-count contract" >&2
  exit 1
fi

if [[ -n "$output_json" ]]; then
  cp "$BRIDGE_OUTBOUND_INTENT_DEEP_REPORT" "$output_json"
fi

echo "bridge credentialed deep lane tests passed."
