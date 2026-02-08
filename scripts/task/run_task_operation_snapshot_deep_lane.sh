#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/task/run_task_operation_snapshot_contract_lane.sh"

bash "$CONTRACT_LANE"

cargo test -p kamn-core --lib task_operations::tests::performance_task_operation_snapshot_store_deep_lane_stress -- --ignored >/dev/null

echo "task operation snapshot deep lane tests passed."
