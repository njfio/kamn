# Objective
Add a public high-level SDK service observability trait so callers can read service health and raw metrics through an explicit capability boundary without depending on `ServiceApiClient` internals.

# Inputs/Outputs
- Input: none for health query.
- Output: stable SDK `ServiceHealthSnapshot`.
- Input: none for metrics query.
- Output: raw metrics exposition text.

# Boundaries/Non-goals
- Do not add or modify service routes.
- Do not add an in-memory observability implementation.
- Do not parse or aggregate metrics beyond returning raw exposition text.
- Do not modify `KamnAgent`.

# Failure modes
- Malformed health payloads must continue to fail closed as transport failures.
- Metrics route transport failures must surface unchanged.
- In-memory clients must remain outside the service observability trait surface.

# Acceptance criteria
- [x] Public SDK trait `KamnServiceObservability` exists and exposes health and metrics methods.
- [x] `LiveTransportKamnClient` implements the trait through existing live route-backed behavior.
- [x] Trait methods return stable SDK outputs and do not expose low-level `ServiceApiClient` response structs.
- [x] Live SDK tests exercise the trait methods through real `/healthz` and `/metrics` route calls.
- [x] `InMemoryKamnClient` does not implement the trait.

# Files to touch
- `crates/kamn-sdk/src/agent.rs` or dedicated trait module if extraction is cleaner
- `crates/kamn-sdk/src/lib.rs`
- `crates/kamn-sdk/src/live.rs` and/or live observability helper seam as needed
- `crates/kamn-sdk/tests/live_transport_observability.rs`
- `specs/6610-add-sdk-service-observability-trait.md`

# Error semantics
- Trait methods must preserve `SdkError::TransportFailure` for malformed health payloads.
- Trait methods must preserve existing `SdkError::ServiceApiError` mappings for non-success responses.
- Trait methods must preserve transport/connectivity failures unchanged.

# Test plan
- Add red tests that call the new public trait methods through `LiveTransportKamnClient`.
- Keep the malformed health fail-closed regression on the trait method path.
- Re-run the existing live observability route contract test.
- Re-run the full `kamn-sdk` test suite and strict `clippy`.

# Deviations
- None.

# Verification
- `cargo test -p kamn-sdk --test live_transport_observability -- --nocapture`
- `cargo test -p kamn-sdk -- --nocapture`
- `cargo clippy -p kamn-sdk --tests -- -D warnings`
