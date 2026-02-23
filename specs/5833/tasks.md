# Tasks: Issue #5833 - Live S-13 Bridge-Forwarding Activation

- Issue: #5833
- Spec: `specs/5833/spec.md`
- Plan: `specs/5833/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing `S-13` fail-closed driver tests for sdk/cli/mcp.
- [x] T2 (GREEN/API Surface): implement deterministic S-13 operation contracts across service API + SDK + agent-lib.
- [x] T3 (GREEN/CLI+MCP): add S-13 CLI commands and MCP tools/dispatch coverage.
- [x] T4 (GREEN/Drivers): wire `S-13` live routing and implement per-driver bridge-forwarding probes.
- [x] T5 (Regression): execute targeted suites and full harness/workspace regressions.
- [x] T6 (Quality Gates): run fmt/clippy/docs-contract/mutation/workspace gates.
- [x] T7 (Lifecycle): update milestone slice markers and finalize lifecycle statuses.

## Tier Mapping
- Unit: per-surface field validation and parser/mapper coverage for S-13 operations.
- Functional: driver `execute("S-13")` fail-closed conformance tests.
- Integration: service API + CLI + MCP + harness package and workspace lanes.
- Regression: existing scenario/toggle/docs-contract lanes remain green.
- Performance: N/A (no hotspot/runtime algorithm change).
