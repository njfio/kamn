#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPLAY_FIXTURE="$ROOT_DIR/fixtures/bridge_replay/replay_validation_cases.json"
REPLAY_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_matrix.sh"
REDACTION_CHECK_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credential_redaction_check.py"
OUTBOUND_INTENT_CONTRACT_SCRIPT="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_contract_lane.sh"

skip_replay=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-replay)
      skip_replay=true
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BRIDGE_REPLAY_REPORT="$TMP_DIR/bridge-replay-report.json"
BRIDGE_CREDENTIAL_REDACTION_REPORT="$TMP_DIR/bridge-credential-redaction-report.json"
BRIDGE_REDACTION_STDOUT="$TMP_DIR/bridge-redaction.out"

if [ "$skip_replay" != true ]; then
  bash "$REPLAY_SCRIPT" \
    --fixture "$REPLAY_FIXTURE" \
    --suites "bridge_adapter,discord_bridge,cross_chain_bridge" \
    --output-json "$BRIDGE_REPLAY_REPORT" >/dev/null
fi

bash "$OUTBOUND_INTENT_CONTRACT_SCRIPT" >/dev/null

TELEGRAM_TOKEN="telegram_contract_token_q7h3n5m1v9x2"
DISCORD_TOKEN="discord_contract_token_z6k4d8r2p1w5"
CROSS_CHAIN_TOKEN="crosschain_contract_token_c9m4x2r7t1q6"

python3 "$REDACTION_CHECK_SCRIPT" \
  --mode contract \
  --output-json "$BRIDGE_CREDENTIAL_REDACTION_REPORT" \
  --telegram-token "$TELEGRAM_TOKEN" \
  --discord-token "$DISCORD_TOKEN" \
  --cross-chain-token "$CROSS_CHAIN_TOKEN" >"$BRIDGE_REDACTION_STDOUT"

if ! grep -q '^status=pass$' "$BRIDGE_REDACTION_STDOUT"; then
  echo "expected redaction contract checker to pass" >&2
  exit 1
fi

for secret in "$TELEGRAM_TOKEN" "$DISCORD_TOKEN" "$CROSS_CHAIN_TOKEN"; do
  if grep -q "$secret" "$BRIDGE_CREDENTIAL_REDACTION_REPORT"; then
    echo "redaction report leaked raw credential material" >&2
    exit 1
  fi
  if grep -q "$secret" "$BRIDGE_REDACTION_STDOUT"; then
    echo "redaction checker output leaked raw credential material" >&2
    exit 1
  fi
done

echo "bridge credentialed contract lane tests passed."
