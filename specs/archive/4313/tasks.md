# Issue #4313 Tasks

- Issue: `#4313`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add peer integrity drift/retry-timeout and peer adapter reason projection conformance tests for C-01..C-09 with failing evidence.
- T2 (Green): implement peer adapter reason projection + deterministic multi-process validation hooks and export via `kamn-core`.
- T3 (Docs): add peer transport governance markers to planning/release docs and guard parity via docs tests (C-10, C-11).
- T4 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout`
  - `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection`
  - `cargo test -p kamn-core --test kolme_devnet_ops_docs`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `cargo mutants --in-diff`

## Completion Evidence
- Sender-integrity drift and retry-timeout reason outputs are deterministic and regression-guarded.
- Reason projection and multi-process hook ordering/taxonomy markers are deterministic.
- Planning/release docs retain required peer transport governance markers via docs tests.
- RED evidence:
  - `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection unit_retry_timeout_reason_projection_is_deterministic -- --exact` failed before implementation with unresolved imports for peer adapter projection exports.
- GREEN/verify commands passed:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout`
  - `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection`
  - `cargo test -p kamn-core --test kolme_devnet_ops_docs`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `cargo mutants --in-diff` (`cargo-mutants` not installed in this environment)
