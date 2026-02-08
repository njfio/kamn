#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credential_redaction_check.py"

if [ ! -f "$SCRIPT" ]; then
  echo "expected bridge credential redaction checker script to exist" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

CONTRACT_REPORT="$TMP_DIR/contract-report.json"
CONTRACT_STDOUT="$TMP_DIR/contract.out"

TELEGRAM_TOKEN="telegram_test_secret_123456"
DISCORD_TOKEN="discord_test_secret_654321"
CROSS_CHAIN_TOKEN="crosschain_test_secret_246810"

python3 "$SCRIPT" \
  --mode contract \
  --output-json "$CONTRACT_REPORT" \
  --telegram-token "$TELEGRAM_TOKEN" \
  --discord-token "$DISCORD_TOKEN" \
  --cross-chain-token "$CROSS_CHAIN_TOKEN" >"$CONTRACT_STDOUT"

if ! grep -q '^status=pass$' "$CONTRACT_STDOUT"; then
  echo "expected contract mode redaction checker to pass" >&2
  exit 1
fi

if ! grep -q '"regression_guard": "Regression: #621"' "$CONTRACT_REPORT"; then
  echo "expected regression marker for credential leakage policy guard" >&2
  exit 1
fi

for secret in "$TELEGRAM_TOKEN" "$DISCORD_TOKEN" "$CROSS_CHAIN_TOKEN"; do
  if grep -q "$secret" "$CONTRACT_REPORT"; then
    echo "contract mode report leaked raw credential material" >&2
    exit 1
  fi
done

DEEP_REPORT="$TMP_DIR/deep-report.json"
DEEP_STDOUT="$TMP_DIR/deep.out"
python3 "$SCRIPT" \
  --mode deep \
  --output-json "$DEEP_REPORT" \
  --telegram-token "$TELEGRAM_TOKEN" \
  --discord-token "$DISCORD_TOKEN" \
  --cross-chain-token "$CROSS_CHAIN_TOKEN" >"$DEEP_STDOUT"

if ! grep -q '^status=pass$' "$DEEP_STDOUT"; then
  echo "expected deep mode redaction checker to pass" >&2
  exit 1
fi

if ! grep -q '"sample_count": 128' "$DEEP_REPORT"; then
  echo "expected deep mode report to include sample-count contract" >&2
  exit 1
fi

set +e
empty_token_output="$(
  python3 "$SCRIPT" \
    --mode contract \
    --output-json "$TMP_DIR/empty-token.json" \
    --telegram-token "" \
    --discord-token "$DISCORD_TOKEN" \
    --cross-chain-token "$CROSS_CHAIN_TOKEN" 2>&1
)"
empty_token_code=$?
set -e

if [ "$empty_token_code" -eq 0 ]; then
  echo "expected redaction checker to fail for empty token" >&2
  exit 1
fi

if ! printf '%s\n' "$empty_token_output" | grep -q "reason=empty-token:telegram"; then
  echo "expected explicit empty-token failure output for telegram connector" >&2
  exit 1
fi

echo "bridge credential redaction checker tests passed."
