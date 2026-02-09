#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

# Keep concurrency lane deterministic and bounded for PR-fast coverage.
cargo test -p kamn-core --test concurrency_state_mutation unit_concurrency_replay_fixture_entries_are_valid -- --exact >/dev/null
cargo test -p kamn-core --test concurrency_state_mutation task_accept_concurrency_has_single_winner_and_consistent_state -- --exact >/dev/null
cargo test -p kamn-core --test concurrency_state_mutation task_submit_concurrency_rejects_duplicate_task_id_deterministically -- --exact >/dev/null
cargo test -p kamn-core --test concurrency_state_mutation peer_lifecycle_concurrency_preserves_transition_contract_across_phases -- --exact >/dev/null
cargo test -p kamn-core --test concurrency_state_mutation functional_task_accept_concurrency_replay_fixture_preserves_invariants -- --exact >/dev/null
cargo test -p kamn-core --test concurrency_state_mutation integration_peer_lifecycle_concurrency_replay_is_deterministic_across_rounds -- --exact >/dev/null
cargo test -p kamn-core --test concurrency_state_mutation functional_escrow_dispute_refund_concurrency_replay_fixture_preserves_terminal_snapshot -- --exact >/dev/null
cargo test -p kamn-core --test concurrency_state_mutation integration_escrow_dispute_refund_concurrency_replay_is_deterministic_across_rounds -- --exact >/dev/null
cargo test -p kamn-core --test concurrency_state_mutation regression_concurrency_accept_race_never_allows_multiple_winners -- --exact >/dev/null
cargo test -p kamn-core --test concurrency_state_mutation regression_escrow_refund_race_never_allows_multiple_refund_winners -- --exact >/dev/null
cargo test -p kamn-core --test concurrency_state_mutation performance_concurrency_state_mutation_contract_lane_stays_within_budget -- --exact >/dev/null
cargo test -p kamn-core --test concurrency_state_mutation performance_escrow_dispute_refund_concurrency_lane_stays_within_budget -- --exact >/dev/null

cargo test -p kamn-core --test runtime_network_docs doc_contains_concurrency_harness_contract_rules -- --exact >/dev/null

echo "runtime concurrency state mutation contract lane tests passed."
