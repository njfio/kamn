# Issue #4096 Tasks

## Ordered Task List
- [x] T1 (Tests/RED): Add `scripts/ci/test_check_daemon_os_signal_stress_policy.sh` and initial `ci_strategy_docs` assertions; run and confirm failure before checker/docs updates.
- [x] T2 (Implementation): Add `fixtures/ci/daemon_os_signal_stress_policy_thresholds.env` baseline fixture.
- [x] T3 (Implementation): Add `scripts/ci/check_daemon_os_signal_stress_policy.sh` fail-closed checker.
- [x] T4 (Integration): Wire checker regression script into `scripts/ci/test_ci_tools.sh` fast/full paths.
- [x] T5 (Docs): Update `docs/ci/strategy.md` with checker command and marker contract block.
- [x] T6 (GREEN/Regression): Run targeted checker tests, docs-contract tests, and formatting checks.
- [ ] T7 (Closure): Update issue log + labels and open PR with AC mapping and shell-surface actuals.

## Tier Mapping
- Unit: fixture/threshold parsing checks in checker test
- Functional: checker pass/fail behavior
- Integration: selector command-surface guard composition
- Regression: threshold drift and heavy-run leakage failures
- Performance: runtime budget check marker validation
