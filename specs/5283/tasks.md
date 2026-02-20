# Issue #5283 Tasks

## Ordered Execution (TDD-first)
- [x] T1 (Red): add failing bounded-load/realtime regression tests for backpressure, anti-spam, ordering, and duplicate suppression (`C-01`..`C-05`).
- [x] T2 (Green): implement minimal websocket realtime handling changes to satisfy deterministic guardrail outcomes and stable reason markers (`C-01`..`C-05`).
- [x] T3 (Docs): update `docs/ops/configuration.md` with presence-mode required headers, owner-scope constraints, and fail-closed troubleshooting map (`C-06`).
- [x] T4 (Verify): run `cargo fmt --check`, strict `clippy` for `kamn-node`, and targeted test commands (`C-07`).
- [x] T5 (Closeout): update issue/plan status markers, prepare PR AC mapping, and include shell-surface delta markers.

## Tier Mapping
- Unit: helper parsing/decision utility tests in websocket path.
- Functional: API route behavior assertions for fail-closed reason markers.
- Integration: realtime websocket flow under bounded pressure.
- Regression: ordering + duplicate suppression + anti-spam breach scenarios.
- Performance: bounded perf-smoke contract lane for realtime slice.

## Dependencies
- Parent story: `#5252`
- Prior merged tasks: `#5279`, `#5281`
