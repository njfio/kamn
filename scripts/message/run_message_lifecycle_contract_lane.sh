#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
KEY_HIERARCHY_LANE="$ROOT_DIR/scripts/message/run_key_hierarchy_invariant_contract_lane.sh"
GROUP_REPLAY_RATCHET_LANE="$ROOT_DIR/scripts/message/run_group_sender_replay_ratchet_contract_lane.sh"
DIDCOMM_COMPAT_LANE="$ROOT_DIR/scripts/message/run_didcomm_envelope_compatibility_contract_lane.sh"
A2A_MCP_CONFORMANCE_LANE="$ROOT_DIR/scripts/message/run_a2a_mcp_conformance_contract_lane.sh"

cargo test -p kamn-core --lib message_lifecycle::tests:: >/dev/null
cargo test -p kamn-core --test message_lifecycle_docs >/dev/null
bash "$KEY_HIERARCHY_LANE" >/dev/null
bash "$GROUP_REPLAY_RATCHET_LANE" >/dev/null
bash "$DIDCOMM_COMPAT_LANE" --skip-tests >/dev/null
bash "$A2A_MCP_CONFORMANCE_LANE" --skip-tests >/dev/null

echo "message lifecycle snapshot contract lane tests passed."
