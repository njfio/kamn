# Issue #4479 Tasks

- Issue: `#4479`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add docs parity test for merge-gate reliability CI smoke/local-heavy boundary markers and capture failing evidence.
- T2 (Green): implement deterministic reliability taxonomy/normalized outputs and fail-closed CI smoke/local-heavy boundary checks in anti-flake checker.
- T3 (Regression): extend anti-flake checker tests for workflow boundary drift and deterministic reason mapping.
- T4 (Docs): update CI strategy docs and parity contract tests.
- T5 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `bash scripts/ci/test_check_anti_flake_policy.sh`
  - `bash scripts/ci/test_anti_flake_merge_gate_policy.sh`
  - `bash scripts/ci/test_ci_strategy_contract.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs doc_contains_merge_gate_reliability_ci_smoke_local_heavy_boundary_markers -- --exact`
  - `cargo mutants --in-diff`

## Completion Evidence
- Deterministic merge-gate reliability reason taxonomy/version and normalized reason markers (`csv/value/class`) were added to anti-flake policy outputs.
- CI smoke/local-heavy boundary governance is fail-closed and emits deterministic boundary violation reasons.
- RED evidence:
  - `cargo test -p kamn-core --test ci_strategy_docs doc_contains_merge_gate_reliability_ci_smoke_local_heavy_boundary_markers -- --exact` failed before implementation because new merge-gate reliability marker strings were missing from `docs/ci/strategy.md`.
- GREEN/verify commands passed:
  - `bash scripts/ci/test_check_anti_flake_policy.sh`
  - `bash scripts/ci/test_anti_flake_merge_gate_policy.sh`
  - `bash scripts/ci/test_ci_strategy_contract.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs doc_contains_merge_gate_reliability_ci_smoke_local_heavy_boundary_markers -- --exact`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo mutants --in-diff` (`cargo-mutants` not installed in this environment)
