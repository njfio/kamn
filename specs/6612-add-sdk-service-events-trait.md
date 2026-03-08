# Objective
Add a public high-level SDK service-events trait so callers can read one event frame from the live service websocket route without constructing `ServiceRequestAuth` or depending on `ServiceApiClient` internals.

# Inputs/Outputs
- Input: none for one-shot event read.
- Output: stable SDK `ServiceEventSnapshot` containing event name, runtime mode, role, and sequence.

# Boundaries/Non-goals
- Do not add or modify service routes or websocket semantics.
- Do not add a streaming iterator or subscription surface beyond one-shot event read.
- Do not add an in-memory service event implementation.
- Do not modify `KamnAgent`.

# Failure modes
- Malformed websocket event payloads must fail closed as transport failures.
- Websocket upgrade failures must preserve existing service error mappings.
- Live connection and auth failures must preserve existing transport/auth failures unchanged.
- In-memory clients must remain outside the service-events trait surface.

# Acceptance criteria
- [ ] Public SDK trait `KamnServiceEvents` exists and exposes a one-shot event read method.
- [ ] `LiveTransportKamnClient` implements the trait through the existing `/v1/events/ws` route.
- [ ] Trait methods return stable SDK outputs and do not require callers to construct `ServiceRequestAuth`.
- [ ] Live SDK tests exercise the trait method through a real websocket upgrade and event frame.
- [ ] Malformed websocket event payloads fail closed on the trait method path.
- [ ] `InMemoryKamnClient` does not implement the trait.

# Files to touch
- `crates/kamn-sdk/src/observability.rs` or a dedicated adjacent event module if extraction is cleaner
- `crates/kamn-sdk/src/lib.rs`
- `crates/kamn-sdk/src/live.rs` and/or a live helper seam as needed
- dedicated live events test under `crates/kamn-sdk/tests/`
- `specs/6612-add-sdk-service-events-trait.md`

# Error semantics
- Trait methods must preserve `SdkError::TransportFailure` for malformed websocket payloads.
- Trait methods must preserve existing `SdkError::ServiceApiError` mappings for websocket upgrade failures.
- Trait methods must preserve transport/connectivity failures unchanged.

# Test plan
- Add red tests that call the new public trait through `LiveTransportKamnClient`.
- Add a malformed event payload fail-closed regression on the trait method path.
- Re-run the existing low-level `ServiceApiClient::read_event_once()` coverage.
- Re-run the full `kamn-sdk` test suite and strict `clippy`.
