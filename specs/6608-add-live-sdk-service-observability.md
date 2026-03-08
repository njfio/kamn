# Objective
Add a live-only SDK service observability surface so callers can read service health and raw metrics through `LiveTransportKamnClient` without dropping to the low-level service client.

# Inputs/Outputs
- Input: none for service health query.
- Output: stable SDK `ServiceHealthSnapshot` containing health, runtime mode, role, and observability markers.
- Input: none for metrics query.
- Output: raw metrics exposition text from the service.

# Boundaries/Non-goals
- Do not add or modify service routes.
- Do not add an in-memory observability surface.
- Do not parse or aggregate metrics beyond returning raw exposition text.
- Do not modify `KamnAgent`.

# Failure modes
- Malformed health payloads must fail closed as transport failures.
- Metrics route transport failures must surface unchanged.
- Live client connection errors must continue to fail closed.

# Acceptance criteria
- [x] `LiveTransportKamnClient` exposes service observability methods for health and metrics.
- [x] Health is returned through a stable SDK type rather than requiring direct use of low-level service models.
- [x] Metrics returns the raw `/metrics` exposition text through the live SDK.
- [x] Live SDK tests exercise real `/healthz` and `/metrics` route calls.
- [x] Existing low-level `ServiceApiClient::health()` and `metrics()` coverage remains green.

# Files to touch
- `crates/kamn-sdk/src/live.rs`
- `crates/kamn-sdk/src/live/` helper seam as needed
- `crates/kamn-sdk/src/lib.rs`
- `crates/kamn-sdk/src/types.rs` or dedicated SDK observability type module
- dedicated live observability test under `crates/kamn-sdk/tests/`
- `specs/6608-add-live-sdk-service-observability.md`

# Error semantics
- Return `SdkError::TransportFailure` when the service health payload is malformed.
- Preserve existing `SdkError::ServiceApiError` mappings for non-200 health or metrics responses.
- Preserve existing transport/connectivity failures unchanged.

# Test plan
- Add red tests for live health and metrics through a real contract server.
- Add a red fail-closed test for malformed health payload.
- Re-run existing signed low-level `ServiceApiClient` health/metrics coverage.
- Run strict `clippy` for `kamn-sdk` and the full `kamn-sdk` test suite.

# Deviations
- None.

# Verification
- `cargo test -p kamn-sdk --test live_transport_observability -- --nocapture`
- `cargo test -p kamn-sdk --test service_api_client functional_service_api_client_executes_signed_http_route_contracts -- --exact --nocapture`
- `cargo test -p kamn-sdk -- --nocapture`
- `cargo clippy -p kamn-sdk --tests -- -D warnings`
