# Tasks: Issue #6007

## Ordered Tasks
- T1 (RED): Add failing conformance tests for runtime relay counter advancement and metrics progression under live daemon processing.
- T2 (Implementation): Ensure daemon tick loop executes relay spool/state processing and updates deterministic per-run counters.
- T3 (Implementation): Ensure service API metrics expose live relay counters from runtime state rather than placeholder values.
- T4 (GREEN): Run targeted runtime/service-api tests for conformance cases C-01..C-03.
- T5 (Regression): Run full-supervisor and existing contract regressions (C-04).

## Tier Mapping
- Unit: T1, T2, T4
- Functional: T1, T2, T3, T4
- Conformance: T1, T4
- Integration: T1, T3, T4
- Regression: T5
- Performance: N/A (no hotspot algorithm change)
