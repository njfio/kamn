#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPLAY_FIXTURE="$ROOT_DIR/fixtures/bridge_replay/replay_validation_cases.json"
REPLAY_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_matrix.sh"

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

if [ "$skip_replay" != true ]; then
  bash "$REPLAY_SCRIPT" \
    --fixture "$REPLAY_FIXTURE" \
    --suites "bridge_adapter,telegram_bridge" \
    --output-json "$TMP_DIR/telegram-ingress-replay-report.json" >/dev/null
fi

cargo test -p kamn-core --test telegram_bridge >/dev/null

echo "telegram ingress contract lane tests passed."
