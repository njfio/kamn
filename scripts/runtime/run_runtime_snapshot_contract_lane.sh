#!/usr/bin/env bash
set -euo pipefail

cargo test -p kamn-core runtime::tests::functional_authenticated_peer_frame_roundtrips_wire_and_signature >/dev/null
cargo test -p kamn-core runtime::tests::regression_forged_or_unauthorized_peer_frame_is_rejected >/dev/null
cargo test -p kamn-core runtime::tests::regression_replayed_peer_frame_nonce_is_rejected >/dev/null
cargo test -p kamn-core runtime::tests::functional_file_snapshot_store_recovery_truncates_stale_metadata_suffix >/dev/null
cargo test -p kamn-core runtime::tests::regression_file_snapshot_store_rejects_cursor_regression_metadata >/dev/null
cargo test -p kamn-core runtime::tests::regression_snapshot_restore_cursor_mismatch_is_rejected >/dev/null
cargo test -p kamn-core --test runtime_network_docs >/dev/null

echo "runtime snapshot contract lane tests passed."
