# Tasks: Issue #5824 - Live S-09 Transport-Failover Activation

- Issue: #5824
- Spec: `specs/5824/spec.md`
- Plan: `specs/5824/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing `S-09` fail-closed driver tests for sdk/cli/mcp and update live-toggle non-live scenario assertions.
- [x] T2 (GREEN/Implementation): wire `S-09` live routing and implement per-driver transport-failover probes with deterministic fail-closed validations.
- [x] T3 (Regression): execute targeted unit/conformance lanes and full harness regression.
- [x] T4 (Quality Gates): run fmt/clippy/docs-contract/mutation/workspace gates.
- [x] T5 (Lifecycle): update milestone slice markers, preserve specs cap, and finalize lifecycle statuses.

## Tier Mapping
- Unit: helper validation branches for failover continuity and response-shape checks.
- Functional: driver `execute("S-09")` fail-closed conformance tests.
- Integration: harness package and workspace integration test lanes.
- Regression: existing scenario/toggle/docs-contract lanes remain green.
- Performance: N/A (no hotspot/runtime algorithm change).
