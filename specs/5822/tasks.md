# Tasks: Issue #5822 - Live S-08 Node-Crash-Recovery Activation

- Issue: #5822
- Spec: `specs/5822/spec.md`
- Plan: `specs/5822/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing `S-08` fail-closed driver tests for sdk/cli/mcp and update live-toggle non-live scenario assertions.
- [x] T2 (GREEN/Implementation): wire `S-08` live routing and implement per-driver crash-recovery continuity probes with deterministic fail-closed validations.
- [x] T3 (Regression): execute targeted unit/conformance lanes and full harness regression.
- [x] T4 (Quality Gates): run fmt/clippy/docs-contract/mutation/workspace gates.
- [x] T5 (Lifecycle): update milestone slice markers, preserve specs cap, and finalize lifecycle statuses.

## Tier Mapping
- Unit: helper validation branches for continuity and response-shape checks.
- Functional: driver `execute("S-08")` fail-closed conformance tests.
- Integration: harness package and workspace integration test lanes.
- Regression: existing scenario/toggle/docs-contract lanes remain green.
- Performance: N/A (no hotspot/runtime algorithm change).
