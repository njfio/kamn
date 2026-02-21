#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$ROOT_DIR"

cargo test -p kamn-core --test cross_store_replay_consistency -- --nocapture

echo "cross_store_replay_consistency_policy_status=verified"
echo "cross_store_replay_consistency_contract_lane_status=verified"
echo "cross-store replay consistency contract lane tests passed."
