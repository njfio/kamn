# Tasks: Issue #5814 - Live S-03 Scenario Activation

- Issue: #5814
- Spec: `specs/5814/spec.md`
- Plan: `specs/5814/plan.md`
- Status: Done

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing `S-03` fail-closed driver tests for sdk/cli/mcp.
- [x] T2 (GREEN/Implementation): wire `S-03` live-bound routing and implement per-driver S-03 live helpers.
- [x] T3 (Regression): update/verify live-toggle contracts and run full harness regression.
- [x] T4 (Lifecycle): update milestone slice markers and finalize issue lifecycle files.
- [x] T5 (Quality Gates): run fmt/clippy/scoped guards including spec-volume non-regression lane.

## Tier Mapping
- Unit: env/helper failure-path checks in driver modules.
- Functional: driver `execute("S-03")` fail-closed conformance tests.
- Integration: harness crate integration path via full test suite.
- Regression: existing scenario/toggle lanes remain green.
- Performance: N/A (no hotspot/runtime algorithm changes).
