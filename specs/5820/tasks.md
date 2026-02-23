# Tasks: Issue #5820 - Live S-07 Replay-Protection Activation

- Issue: #5820
- Spec: `specs/5820/spec.md`
- Plan: `specs/5820/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing `S-07` fail-closed driver tests for sdk/cli/mcp.
- [x] T2 (GREEN/Implementation): wire `S-07` live routing and implement per-driver replay probes with deterministic reason checks.
- [x] T3 (Regression): update live-toggle contracts and run full harness regression.
- [x] T4 (Quality Gates): run fmt/clippy/docs-contract/mutation/workspace gates.
- [x] T5 (Lifecycle): update milestone slice markers and finalize lifecycle statuses.

## Tier Mapping
- Unit: helper validation branches for replay-reason parsing and failure-path checks.
- Functional: driver `execute("S-07")` fail-closed conformance tests.
- Integration: harness package and workspace integration test lanes.
- Regression: existing scenario/toggle/docs-contract lanes remain green.
- Performance: N/A (no hotspot/runtime algorithm change).
