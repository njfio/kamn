# Issue #4095 Tasks

## Ordered Task List
- [x] T1 (Tests/RED): Extend overload checker tests to require report taxonomy markers and add failing taxonomy-drift scenarios.
- [x] T2 (Implementation): Emit overload taxonomy markers from stress matrix report JSON/stdout and add threshold keys.
- [x] T3 (Implementation): Enforce report taxonomy/version and reason-csv drift checks in `check_daemon_os_signal_stress_policy.py`.
- [x] T4 (Integration): Update CI strategy marker contract documentation for new threshold keys and mismatch reason markers.
- [x] T5 (GREEN/Regression): Run targeted overload shell tests and docs-contract tests until fully green.
- [ ] T6 (Closure): Update issue status/logs, open PR with AC/test mapping, merge, and close parent hierarchy items if complete.

## Tier Mapping
- Unit: threshold key parsing and validation cases in checker tests.
- Functional: runner/checker pass path with valid markers.
- Integration: runner + checker + fixture composition.
- Regression: taxonomy-version and reason-csv mismatch fail-closed checks.
- Performance: checker runtime remains bounded and CI-fast compatible.
