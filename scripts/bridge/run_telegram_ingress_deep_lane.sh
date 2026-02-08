#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPLAY_FIXTURE="$ROOT_DIR/fixtures/bridge_replay/replay_validation_cases.json"
REPLAY_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_matrix.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/bridge/run_telegram_ingress_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

TELEGRAM_INGRESS_REPORT="$TMP_DIR/telegram-ingress-report.json"

bash "$CONTRACT_LANE" --skip-replay

bash "$REPLAY_SCRIPT" \
  --fixture "$REPLAY_FIXTURE" \
  --suites "bridge_adapter,telegram_bridge" \
  --output-json "$TELEGRAM_INGRESS_REPORT" >/dev/null

cargo test -p kamn-core --test telegram_bridge_listener_docs >/dev/null

echo "telegram ingress deep lane tests passed."
