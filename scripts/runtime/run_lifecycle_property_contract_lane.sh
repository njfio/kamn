#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

# Keep the lane bounded to exact property invariants needed for task/escrow/peer lifecycles.
cargo test -p kamn-core --test task_state_machine task_lifecycle_property_generated_sequences_preserve_transition_contracts -- --exact >/dev/null
cargo test -p kamn-core --test task_state_machine task_lifecycle_property_restore_roundtrip_preserves_state_and_history -- --exact >/dev/null
cargo test -p kamn-core --test task_state_machine task_lifecycle_property_terminal_states_are_absorbing -- --exact >/dev/null

cargo test -p kamn-core --test escrow_lifecycle escrow_property_generated_action_sequences_preserve_amount_and_status_invariants -- --exact >/dev/null
cargo test -p kamn-core --test escrow_lifecycle escrow_property_terminal_statuses_reject_all_mutating_actions -- --exact >/dev/null

cargo test -p kamn-core --test runtime_peer_lifecycle peer_lifecycle_property_generated_event_sequences_match_transition_contract -- --exact >/dev/null
cargo test -p kamn-core --test runtime_peer_lifecycle peer_lifecycle_property_sequence_replay_is_deterministic -- --exact >/dev/null
cargo test -p kamn-core --test runtime_peer_lifecycle peer_lifecycle_property_roundtrip_disconnect_recovers_connection_path -- --exact >/dev/null

echo "runtime lifecycle property contract lane tests passed."
