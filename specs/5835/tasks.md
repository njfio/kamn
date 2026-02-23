# Tasks: Issue #5835 - Live S-14 Batch-Merkle Activation

- Issue: #5835
- Spec: `specs/5835/spec.md`
- Plan: `specs/5835/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing `S-14` fail-closed driver tests for sdk/cli/mcp.
- [x] T2 (GREEN/Drivers): wire `S-14` live routing and add per-driver batch-merkle probes.
- [x] T3 (GREEN/Probe Validation): add focused probe guard tests (invalid endpoint/missing binary) for S-14.
- [x] T4 (Regression): execute targeted suites plus full e2e-harness regression lane.
- [x] T5 (Quality Gates): run docs-contract lane and ensure workspace remains green.
- [x] T6 (Lifecycle): update milestone slice markers and finalize lifecycle statuses.

## Tier Mapping
- Unit: probe-level field validation and error-path guards.
- Functional: driver `execute("S-14")` fail-closed conformance tests.
- Integration: harness execution against existing send/query/verify surfaces.
- Conformance: spec C-01..C-06 coverage.
- Regression: full harness/docs-contract lanes remain green.
- Performance: N/A (no hotspot/runtime algorithm change).
