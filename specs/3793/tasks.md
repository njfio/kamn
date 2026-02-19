# Issue #3793 Tasks

- Issue: #3793
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add failing tests/docs-contract assertions for reconnect pacing policy markers and pacing behavior.
- [x] T2 (Green): implement deterministic reconnect pacing schedule with bounded cap in notifications consumer loop.
- [x] T3 (Functional/Integration): verify reconnect loop behavior and websocket integration paths remain green.
- [x] T4 (Regression/Performance): verify retry exhaustion stability and reconnect pacing budget constraints.
- [ ] T5 (Verify): run fmt/clippy/shell guardrails, open mergeable PR, and close issue.

## Tier Mapping
- Unit: reconnect pacing helper schedule checks.
- Functional: reconnect loop under repeated disconnect/connect failures.
- Integration: websocket connector notification receipt behavior.
- Regression: reconnect exhaustion reason stability + docs drift checks.
- Performance: bounded reconnect pacing upper-budget checks.
