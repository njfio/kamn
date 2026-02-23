# Tasks: Issue #5837 - Live S-15 Performance-Smoke Activation

- Issue: #5837
- Spec: `specs/5837/spec.md`
- Plan: `specs/5837/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing S-15 fail-closed driver tests for sdk/cli/mcp.
- [x] T2 (GREEN/Drivers): wire S-15 live routing and add per-driver performance-smoke probes.
- [x] T3 (GREEN/Validation): add S-15 latency-budget validator tests (p50/p99/total pass+fail).
- [x] T4 (Regression): execute targeted S-15 suites and full harness regression lane.
- [x] T5 (Quality Gates): run docs-contract lane, mutation in-diff, fmt/clippy, and workspace tests.
- [x] T6 (Lifecycle): update milestone slice markers and finalize lifecycle statuses.

## Tier Mapping
- Unit: S-15 latency-budget validation helpers and branch checks.
- Functional: driver `execute("S-15")` fail-closed tests.
- Integration: live S-15 probe flow over existing send/query surfaces.
- Conformance: spec C-01..C-07 coverage.
- Regression: full harness/docs-contract lanes remain green.
- Performance: covered directly by S-15 bounded-latency validation contracts.
