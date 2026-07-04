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

bridge_ingress_relay_target_dir="${KAMN_BRIDGE_INGRESS_RELAY_TARGET_DIR:-$ROOT_DIR/target/bridge-ingress-relay-contract}"
mkdir -p "$bridge_ingress_relay_target_dir"

start_epoch="$(date +%s)"

if [ "$skip_replay" != true ]; then
  bash "$REPLAY_SCRIPT" \
    --fixture "$REPLAY_FIXTURE" \
    --suites "bridge_adapter,telegram_bridge,discord_bridge" \
    --output-json "$TMP_DIR/bridge-ingress-replay-report.json" >/dev/null
fi

CARGO_TARGET_DIR="$bridge_ingress_relay_target_dir" cargo test -p kamn-core --test bridge_ingress_relay_harness >/dev/null

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 120 ]; then
  echo "bridge ingress relay contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "bridge ingress relay contract lane tests passed."
