# Issue #3792 Tasks

- Issue: #3792
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add failing reconnect terminal taxonomy marker assertions in policy and docs contract tests.
- [x] T2 (Green): implement deterministic reconnect terminal reason-code/taxonomy marker composition.
- [x] T3 (Functional/Integration): verify notifications consumer reconnect exhaustion behavior and integration tests remain green.
- [x] T4 (Regression): verify docs drift checks, policy contract tests, lint, and shell guardrails.
- [ ] T5 (Verify): open mergeable PR and close issue with DoD markers.

## Tier Mapping
- Unit: reconnect terminal reason composition markers.
- Functional: reconnect exhaustion terminal reason-marker behavior.
- Integration: websocket/notifications consumer integration tests.
- Regression: docs + policy taxonomy drift checks and guardrails.
- Performance: N/A (no new algorithmic path; only deterministic reason-marker augmentation).
