# Tasks: Issue #5901 - Replace Fake SDK Live Transport with Network-Backed Behavior

1. T1 (RED, Conformance): replace existing live transport tests with failing tests that require loopback network requests and reject in-memory simulation assumptions.
2. T2 (GREEN, Implementation): refactor `live.rs` to remove global registry/mutex-based `InMemoryKamnClient` wiring.
3. T3 (GREEN, Implementation): implement network-backed `send`, `resolve`, `get_reputation` with Service API auth/signature envelope.
4. T4 (GREEN, Implementation): enforce explicit `SdkError::NotImplemented` for unsupported `KamnAgent` methods in live mode.
5. T5 (REFACTOR): add deterministic service-id to numeric-id mapping helper with collision checks and focused unit coverage.
6. T6 (VERIFY, Conformance): run targeted SDK live transport tests and strict lint/format gates for changed crate.
