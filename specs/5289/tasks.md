# Issue #5289 Tasks

## Ordered Execution (TDD-first)
- [x] T1 (Red): add failing conformance tests for orchestration happy path, deterministic ordering, legal-hold fail-closed, and invalid projection input (`C-01`..`C-06`).
- [x] T2 (Green): implement deterministic Phase-6 orchestration request/report contracts and execution function composing M8 + M10 (`C-01`..`C-04`).
- [x] T3 (Regression): implement explicit fail-closed error mapping and stable reason markers for legal-hold/projection failures (`C-05`..`C-06`).
- [x] T4 (Verify): run `cargo fmt --check`, strict clippy, and targeted orchestration test suite (`C-07`).
- [x] T5 (Closeout): open PR with AC mapping, RED/GREEN evidence, and shell-surface markers.

## Tier Mapping
- Unit: deterministic ordering and zero-due counters.
- Functional: end-to-end retention -> shred -> projection -> archive path.
- Integration: composed M8+M10 registry execution.
- Regression: legal-hold and invalid projection input fail-closed.

## Dependencies
- Parent story: `#5253`
- Prior Phase-6 tasks: `#5285`, `#5287`
