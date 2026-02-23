# Tasks: Issue #5797 - Execute Live Harness S-01/S-04/S-06 Across SDK/CLI/MCP Modes

- Issue: #5797
- Spec: `specs/5797/spec.md`
- Plan: `specs/5797/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (Conformance/RED): execute live harness matrix commands for `sdk-direct`, `cli-scripted`, and `mcp-tau` with `S-01,S-04,S-06`; capture raw outcomes under `.tmp/5797-live/`.
- [x] T2 (GREEN): resolve any execution blockers with minimal changes and rerun until outcomes are deterministic (or record blocker if external prerequisite prevents pass).
- [x] T3 (Docs): publish deterministic live-execution evidence artifact with commands, scenario outcomes, and blocker/prereq notes.
- [x] T4 (Regression): preserve spec-volume cap while adding `specs/5797`, then run R50/R53 docs-contract non-regression tests.
- [x] T5 (Closeout): finalize milestone metadata and lifecycle statuses (`spec=Implemented`, `tasks=Completed`) and post issue process-log updates.

## Tier Mapping
- Unit: N/A (execution/documentation task; no new unit-level behavior introduced).
- Functional: per-mode outcome extraction and status-marker integrity checks.
- Conformance: S-01/S-04/S-06 live run matrix across all target modes.
- Integration: local API runtime plus harness driver interactions for SDK/CLI/MCP paths.
- Regression: R50/R53 docs-contract non-regression cap checks.
