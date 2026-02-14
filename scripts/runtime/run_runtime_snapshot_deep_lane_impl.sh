#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/run_runtime_snapshot_contract_lane.sh"

bash "$CONTRACT_LANE"

cargo test -p kamn-core runtime::tests::performance_file_snapshot_store_recovery_deep_lane_large_payload -- --ignored >/dev/null

echo "runtime snapshot deep lane tests passed."
