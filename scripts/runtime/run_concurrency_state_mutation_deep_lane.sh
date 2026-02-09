#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

bash scripts/runtime/run_concurrency_state_mutation_contract_lane.sh >/dev/null
cargo test -p kamn-core --test concurrency_state_mutation performance_concurrency_state_mutation_deep_lane_stress -- --ignored >/dev/null

echo "runtime concurrency state mutation deep lane tests passed."
