# Issue #4314 Tasks

- Issue: `#4314`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add durable commit replay/tamper and checker projection conformance tests for C-01..C-07 and capture failing evidence.
- T2 (Green): implement durable commit checker reason projection + lane boundary APIs and export via `kamn-core`.
- T3 (Docs): add release go/no-go and CI strategy durable commit governance markers and enforce via docs parity tests.
- T4 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo test -p kamn-core --test block_commit_persistence_tamper_matrix`
  - `cargo test -p kamn-core --test block_commit_checker_reason_mapping`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `cargo test -p kamn-core --test ci_strategy_docs`
  - `cargo mutants --in-diff`

## Completion Evidence
- Replay tamper matrix covers digest mismatch, missing checkpoint, and height drift fail-closed reasons.
- Durable checker reason projection and lane boundary decisions are deterministic and regression-guarded.
- Release checklist and CI strategy durable commit markers are parity-guarded by docs tests.
- RED evidence:
  - `cargo test -p kamn-core --test block_commit_checker_reason_mapping unit_replay_drift_reason_projection_is_deterministic -- --exact` failed before implementation with unresolved imports for durable checker projection exports.
- GREEN/verify commands passed:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo test -p kamn-core --test block_commit_persistence_tamper_matrix`
  - `cargo test -p kamn-core --test block_commit_checker_reason_mapping`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `cargo test -p kamn-core --test ci_strategy_docs`
  - `cargo mutants --in-diff` (`cargo-mutants` not installed in this environment)
