# Issue #5189 Plan

- Issue: #5189
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Approach
1. Add Rust-native replacement suites first:
   - `shell_test_surface_migration_wave1.rs` validates parity for all 20 removed wrappers.
   - `shell_test_surface_ratio_policy.rs` enforces shell-vs-rust test-file ratio non-regression from baseline/threshold fixtures with waiver support.
2. Wire CI to run the Rust migration suite(s) from `scripts/ci/test_ci_tools.sh` in fast and full paths.
3. Update command-surface/doc contracts so they require Rust migration commands instead of deleted shell wrappers.
4. Delete the 20 migrated shell wrappers.
5. Run targeted checks (`cargo fmt`, targeted `cargo test`, command-surface and strategy contracts), then fix any deterministic contract drift.

## Affected Modules / Files
- New:
  - `crates/kamn-core/tests/shell_test_surface_migration_wave1.rs`
  - `crates/kamn-core/tests/shell_test_surface_ratio_policy.rs`
  - `fixtures/ci/shell_test_surface_ratio_baseline.env`
  - `.ci/shell_test_surface_ratio_thresholds.env`
- Updated:
  - `scripts/ci/test_ci_tools.sh`
  - `scripts/ci/test_ci_tools_command_surface_contract.sh`
  - `scripts/ci/test_ci_strategy_contract.sh`
  - `docs/ci/strategy.md`
  - `crates/kamn-core/tests/ci_strategy_docs.rs`
- Deleted:
  - 20 wave-1 wrappers from `scripts/ci` + `scripts/runtime`.

## Risks and Mitigations
- Risk: deletion-wave drift breaks docs/contract tests.
  - Mitigation: update command-surface and strategy docs/tests in same commit.
- Risk: ratio policy creates false positives.
  - Mitigation: deterministic baseline + explicit warn/fail thresholds + scoped waiver schema with mitigation issue marker.
- Risk: CI runtime regression from replacing tiny shell wrappers with too many Rust invocations.
  - Mitigation: use consolidated Rust suites and single-command CI invocation.

## Interfaces / Contracts
- New ratio report schema marker: `kamn.ci.shell-test-surface-ratio-report.v1`
- New ratio reason taxonomy marker: `kamn.ci.shell-test-surface-ratio-reason-taxonomy.v1`
- Deterministic reason codes:
  - `none`
  - `ratio_warn_threshold_exceeded`
  - `ratio_fail_threshold_exceeded_unwaived`
  - `ratio_fail_threshold_waiver_applied`
  - `baseline_file_missing|baseline_file_invalid|threshold_file_missing|threshold_file_invalid|waiver_file_invalid|waiver_scope_mismatch|waiver_missing_mitigation_issue|waiver_invalid_mitigation_issue`
