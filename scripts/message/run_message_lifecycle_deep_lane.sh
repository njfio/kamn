#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/message/run_message_lifecycle_contract_lane.sh"

bash "$CONTRACT_LANE"

cargo test -p kamn-core --lib message_lifecycle::tests::performance_message_lifecycle_snapshot_deep_lane_stress -- --ignored >/dev/null

echo "message lifecycle snapshot deep lane tests passed."
