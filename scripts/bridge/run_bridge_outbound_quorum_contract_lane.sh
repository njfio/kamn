#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTBOUND_INTENT_CONTRACT_LANE="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_contract_lane.sh"

skip_intent_lane=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-intent-lane)
      skip_intent_lane=true
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

start_epoch="$(date +%s)"

if [ "$skip_intent_lane" != true ]; then
  bash "$OUTBOUND_INTENT_CONTRACT_LANE" >/dev/null
fi

cargo test -p kamn-core --test bridge_outbound_quorum_execution >/dev/null

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 120 ]; then
  echo "bridge outbound quorum contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "bridge outbound quorum contract lane tests passed."
