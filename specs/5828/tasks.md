# Tasks: Issue #5828 - Live S-11 Signer-Rotation Activation

- Issue: #5828
- Spec: `specs/5828/spec.md`
- Plan: `specs/5828/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing `S-11` fail-closed driver tests for sdk/cli/mcp and update live-toggle non-live scenario assertions.
- [x] T2 (GREEN/Implementation): wire `S-11` live routing and implement per-driver signer-rotation probes with deterministic fail-closed validations.
- [x] T3 (Regression): execute targeted unit/conformance lanes and full harness regression.
- [x] T4 (Quality Gates): run fmt/clippy/docs-contract/mutation/workspace gates.
- [x] T5 (Lifecycle): update milestone slice markers, preserve specs cap, and finalize lifecycle statuses.

## Tier Mapping
- Unit: helper validation branches for signer continuity/replay rejection checks.
- Functional: driver `execute("S-11")` fail-closed conformance tests.
- Integration: harness package and workspace integration test lanes.
- Regression: existing scenario/toggle/docs-contract lanes remain green.
- Performance: N/A (no hotspot/runtime algorithm change).
