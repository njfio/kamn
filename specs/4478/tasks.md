# Issue #4478 Tasks

- Issue: `#4478`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add anti-flake docs parity/test expectations for deterministic reason taxonomy/rerun-policy markers and capture failing evidence.
- T2 (Green): implement deterministic anti-flake reason taxonomy, normalized outputs, and rerun-policy checks in policy checker.
- T3 (Docs): update CI strategy anti-flake section with reason taxonomy/rerun-policy markers.
- T4 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `bash scripts/ci/test_check_anti_flake_policy.sh`
  - `bash scripts/ci/test_workflow_retry_policy.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs doc_contains_anti_flake_rerun_policy_reason_taxonomy_markers -- --exact`
  - `cargo mutants --in-diff`

## Completion Evidence
- Anti-flake checker emits deterministic reason taxonomy and normalized reason CSV/value markers.
- Rerun-policy drift is detected fail closed with stable reason codes.
- CI strategy docs marker set remains parity-guarded by tests.
- RED evidence:
  - `cargo test -p kamn-core --test ci_strategy_docs doc_contains_anti_flake_rerun_policy_reason_taxonomy_markers -- --exact` failed before implementation because anti-flake/rerun-policy taxonomy markers were missing from `docs/ci/strategy.md`.
- GREEN/verify commands passed:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `bash scripts/ci/test_check_anti_flake_policy.sh`
  - `bash scripts/ci/test_workflow_retry_policy.sh`
  - `bash scripts/ci/test_anti_flake_merge_gate_policy.sh`
  - `bash scripts/ci/test_ci_strategy_contract.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs doc_contains_anti_flake_rerun_policy_reason_taxonomy_markers -- --exact`
  - `cargo mutants --in-diff` (`cargo-mutants` not installed in this environment)
