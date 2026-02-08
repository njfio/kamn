#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/channel/run_channel_lifecycle_contract_lane.sh"

bash "$CONTRACT_LANE"

cargo test -p kamn-core --lib channel_models::tests::performance_channel_snapshot_deep_lane_stress -- --ignored >/dev/null

echo "channel lifecycle snapshot deep lane tests passed."
