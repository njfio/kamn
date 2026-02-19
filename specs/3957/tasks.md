# Issue #3957 Tasks

- Issue: #3957
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add failing quorum decision-path matrix tests in signer unit/integration harnesses.
- [x] T2 (Green): wire matrix fixture helper usage to existing preflight paths and make tests pass.
- [x] T3 (Green): add ops configuration quorum matrix contract markers and docs-contract assertions.
- [x] T4 (Regression): run targeted signer/docs contract suite, `cargo fmt --check`, and `cargo clippy -p kamn-node -- -D warnings`.
- [ ] T5 (Verify): update issue body/labels/process log and parent task/story backlog markers.

## Tier Mapping
- Unit: N/A (coverage lands in functional signer preflight path).
- Functional: signer preflight quorum matrix behavior.
- Integration: main test harness matrix path checks.
- Regression: deterministic reason-marker drift checks.
- Performance: N/A (bounded deterministic matrix).
