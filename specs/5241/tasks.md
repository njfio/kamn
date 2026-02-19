# Issue #5241 Tasks

## Ordered Task List
- [x] T1 (Tests/RED): Add python overload checker regression test implementation and intentionally keep wrapper stale to confirm failure.
- [x] T2 (Implementation): Convert shell test script to thin wrapper over python test.
- [x] T3 (GREEN): Run overload checker regression test and confirm deterministic pass/fail assertions remain intact.
- [x] T4 (Regression): Run `test_ci_tools_command_surface_contract.sh`, shell ratio guardrail, and shell hard ceiling checks.
- [ ] T5 (Closure): Update issue comments/labels and open PR with measured shell/rust delta actuals.

## Tier Mapping
- Functional: checker regression scenarios
- Integration: ci tools command-surface contract
- Regression: shell ratio and hard-ceiling guard checks
