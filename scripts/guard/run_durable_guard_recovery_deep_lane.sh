#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/guard/run_durable_guard_recovery_contract_lane.sh"

bash "$CONTRACT_LANE"

cargo test -p kamn-core --test durable_guard_recovery_matrix performance_durable_guard_recovery_matrix_deep_lane -- --ignored >/dev/null

echo "durable guard recovery deep lane tests passed."
