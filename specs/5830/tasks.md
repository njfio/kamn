# Tasks: Issue #5830 - Live S-12 Retention/Deletion Activation

- Issue: #5830
- Spec: `specs/5830/spec.md`
- Plan: `specs/5830/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing `S-12` fail-closed driver tests for sdk/cli/mcp.
- [x] T2 (GREEN/API Surface): implement deterministic S-12 operation contracts across service API + SDK + agent-lib.
- [x] T3 (GREEN/CLI+MCP): add S-12 CLI commands and MCP tools/dispatch coverage.
- [x] T4 (GREEN/Drivers): wire `S-12` live routing and implement per-driver retention/deletion probes.
- [x] T5 (Regression): execute targeted suites and full harness/workspace regressions.
- [x] T6 (Quality Gates): run fmt/clippy/docs-contract/mutation/workspace gates.
- [x] T7 (Lifecycle): update milestone slice markers, preserve specs cap, and finalize lifecycle statuses.

## Tier Mapping
- Unit: per-surface field validation and parser/mapper coverage for S-12 operations.
- Functional: driver `execute("S-12")` fail-closed conformance tests.
- Integration: service API + CLI + MCP + harness package and workspace lanes.
- Regression: existing scenario/toggle/docs-contract lanes remain green.
- Performance: N/A (no hotspot/runtime algorithm change).
