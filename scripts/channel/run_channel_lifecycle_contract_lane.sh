#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_LANE="$ROOT_DIR/scripts/channel/run_channel_policy_contract_lane.sh"

cargo test -p kamn-core --lib channel_models::tests:: >/dev/null
cargo test -p kamn-core --test channel_models >/dev/null
cargo test -p kamn-core --test channel_models_docs >/dev/null
bash "$POLICY_LANE" >/dev/null

echo "channel lifecycle snapshot contract lane tests passed."
