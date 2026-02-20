# Issue #5287 Tasks

## Ordered Execution (TDD-first)
- [x] T1 (Red): add failing conformance tests for transient retry, capped backoff, exhausted retries, permanent failure, and invalid-policy inputs (`C-01`..`C-05`).
- [x] T2 (Green): implement deterministic M10 archival retry policy projection and reason-marker taxonomy (`C-01`..`C-04`).
- [x] T3 (Regression): ensure invalid configuration and boundary-attempt inputs fail closed deterministically (`C-05`).
- [x] T4 (Verify): run `cargo fmt --check`, strict `clippy`, and targeted M10 test suite (`C-06`).
- [ ] T5 (Closeout): open PR with AC mapping, RED/GREEN evidence, and shell-surface markers.

## Tier Mapping
- Unit: retry policy math + reason-code projection.
- Functional: exhausted/permanent fail-closed outcomes.
- Regression: capped backoff and invalid-policy boundary cases.

## Dependencies
- Parent story: `#5253`
- Prior Phase-6 task: `#5285`
