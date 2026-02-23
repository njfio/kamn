# Tasks: Issue #5818 - Live S-05 Scenario Activation

- Issue: #5818
- Spec: `specs/5818/spec.md`
- Plan: `specs/5818/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing `S-05` fail-closed driver tests for sdk/cli/mcp.
- [x] T2 (GREEN/Implementation): wire `S-05` live-bound routing and implement per-driver S-05 live helpers.
- [x] T3 (Regression): update/verify live-toggle contracts and run full harness regression.
- [x] T4 (Lifecycle): update milestone slice markers and finalize issue lifecycle files.
- [x] T5 (Quality Gates): run fmt/clippy/scoped guards including spec-volume non-regression lane.

## Tier Mapping
- Unit: env/helper failure-path checks in driver modules.
- Functional: driver `execute("S-05")` fail-closed conformance tests.
- Integration: harness crate integration path via full test suite.
- Regression: existing scenario/toggle lanes remain green.
- Performance: N/A (no hotspot/runtime algorithm changes).
