#!/usr/bin/env bash
set -euo pipefail

cargo test -p kamn-core --test durable_guard_recovery_matrix unit_delivery_guard_snapshot_rejects_schema_mismatch >/dev/null
cargo test -p kamn-core --test durable_guard_recovery_matrix unit_channel_policy_snapshot_rejects_schema_mismatch >/dev/null
cargo test -p kamn-core --test durable_guard_recovery_matrix functional_delivery_guard_recovery_restores_nonce_and_replay_state >/dev/null
cargo test -p kamn-core --test durable_guard_recovery_matrix functional_channel_policy_recovery_restores_retention_candidates >/dev/null
cargo test -p kamn-core --test durable_guard_recovery_matrix integration_durable_guard_recovery_matrix_restores_delivery_and_retention_invariants >/dev/null
cargo test -p kamn-core --test durable_guard_recovery_matrix regression_corrupted_delivery_snapshot_rejected_with_explicit_error >/dev/null
cargo test -p kamn-core --test durable_guard_recovery_matrix regression_corrupted_channel_snapshot_rejected_with_explicit_error >/dev/null
cargo test -p kamn-core --test durable_guard_recovery_matrix performance_durable_guard_recovery_contract_lane_budget >/dev/null
cargo test -p kamn-core --test durable_guard_snapshot_store unit_bundle_schema_mismatch_is_rejected >/dev/null
cargo test -p kamn-core --test durable_guard_snapshot_store functional_in_memory_bundle_store_roundtrip >/dev/null
cargo test -p kamn-core --test durable_guard_snapshot_store integration_file_bundle_restore_preserves_invariants >/dev/null
cargo test -p kamn-core --test durable_guard_snapshot_store regression_truncated_bundle_payload_rejected >/dev/null
cargo test -p kamn-core --test durable_guard_snapshot_store performance_bundle_contract_lane_budget >/dev/null
cargo test -p kamn-core --test message_delivery_guards_docs >/dev/null
cargo test -p kamn-core --test channel_permissions_retention_docs >/dev/null
cargo test -p kamn-core --test release_gonogo_checklist_docs >/dev/null

echo "durable guard recovery contract lane tests passed."
