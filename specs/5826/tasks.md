# Tasks: Issue #5826 - Live S-10 Topology-Coherence Activation

- Issue: #5826
- Spec: `specs/5826/spec.md`
- Plan: `specs/5826/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing `S-10` fail-closed driver tests for sdk/cli/mcp and update live-toggle non-live scenario assertions.
- [x] T2 (GREEN/Implementation): wire `S-10` live routing and implement per-driver topology-coherence probes with deterministic fail-closed validations.
- [x] T3 (Regression): execute targeted unit/conformance lanes and full harness regression.
- [x] T4 (Quality Gates): run fmt/clippy/docs-contract/mutation/workspace gates.
- [x] T5 (Lifecycle): update milestone slice markers, preserve specs cap, and finalize lifecycle statuses.

## Tier Mapping
- Unit: helper validation branches for topology query/health continuity checks.
- Functional: driver `execute("S-10")` fail-closed conformance tests.
- Integration: harness package and workspace integration test lanes.
- Regression: existing scenario/toggle/docs-contract lanes remain green.
- Performance: N/A (no hotspot/runtime algorithm change).
