#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
KEY_HIERARCHY_LANE="$ROOT_DIR/scripts/message/run_key_hierarchy_invariant_contract_lane.sh"

cargo test -p kamn-core --lib message_lifecycle::tests:: >/dev/null
cargo test -p kamn-core --test message_lifecycle_docs >/dev/null
bash "$KEY_HIERARCHY_LANE" >/dev/null

echo "message lifecycle snapshot contract lane tests passed."
