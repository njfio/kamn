# Tasks: Issue #6054

## Ordered Tasks
- T1 (RED): Add sqlite-backed state tests that fail unless `.sqlite` paths persist into `SqliteStoreBackend` namespace/key rows.
- T2 (GREEN): Implement state storage backend resolution and shared load/persist helpers in `state_io.rs`.
- T3 (GREEN): Route `ServiceApiMessageStore` and relay projection through backend-aware state I/O.
- T4 (VERIFY): Run `cargo fmt --check`, sqlite-focused service API tests, and relay projection regression tests.

## Tier Mapping
- Unit: T2, T3
- Functional: T1, T3
- Conformance: T1, T4
- Integration: T1, T4
- Regression: T4
- Performance: N/A (storage backend routing only; no throughput behavior changes)
