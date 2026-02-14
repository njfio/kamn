#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_LANE="$ROOT_DIR/scripts/runtime/run_zk_witness_mutation_contract_lane.sh"

bash "$FAST_LANE" >/dev/null
cargo test -p kamn-core --test zk_witness_fuzz_smoke performance_zk_witness_mutation_deep_lane_stress -- --ignored >/dev/null

echo "runtime zk witness mutation deep lane tests passed."
