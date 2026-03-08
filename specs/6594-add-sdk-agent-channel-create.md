# Objective
Add high-level channel creation to the SDK `KamnAgent` API so both in-memory and live clients can obtain `ChannelId` values for channel-scoped messaging through the same top-level workflow surface.

# Inputs/Outputs
- Input: channel name string from a high-level SDK caller.
- Output: `ChannelId` on success.
- Input: live transport channel create request.
- Output: service-backed `ChannelId` derived from `POST /v1/channels/create`.

# Boundaries/Non-goals
- Do not add channel membership, listing, or permissions APIs.
- Do not change the existing service route contract beyond what tests require.
- Do not add channel metadata structs or persistence semantics beyond channel name validation and ID creation.
- Do not change message send/receive behavior except to enable callers to create a channel ID first.

# Failure modes
- Empty or whitespace-only channel names must fail closed as invalid input.
- Live transport auth/signature mismatches must fail closed at the service boundary.
- Invalid or empty service `channel_id` responses must fail closed as transport failures.
- In-memory duplicate channel creation must remain safe and deterministic without corrupting state.

# Acceptance criteria
- [x] `KamnAgent` exposes a channel creation method returning `ChannelId`.
- [x] `InMemoryKamnClient` implements the method with deterministic local semantics.
- [x] `LiveTransportKamnClient` implements the method through the existing `POST /v1/channels/create` route.
- [x] Live transport signs the exact request body sent to the service route.
- [x] SDK tests cover in-memory and live channel creation through the high-level API.
- [x] Existing lower-level `ServiceApiClient::create_channel()` behavior remains green.

# Files to touch
- `crates/kamn-sdk/src/agent.rs`
- `crates/kamn-sdk/src/memory.rs`
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/src/live/routes.rs`
- `crates/kamn-sdk/src/live/state.rs` only if shared auth helpers need reuse
- `crates/kamn-sdk/tests/memory_agent.rs`
- `crates/kamn-sdk/tests/live_transport_agent.rs`
- `crates/kamn-sdk/tests/service_api_client.rs` only if route harness coverage needs extension
- `specs/6594-add-sdk-agent-channel-create.md`

# Error semantics
- Return `SdkError::InvalidInput` for empty/whitespace channel names.
- Return `SdkError::TransportFailure` if the live service response omits or mangles `channel_id`.
- Preserve existing `SdkError::ServiceApiError` mappings for live route auth or HTTP failures.
- Do not silently fabricate fallback channel IDs on live failures.

# Test plan
- Add red tests for high-level in-memory channel creation success and invalid-name rejection.
- Add red tests for high-level live channel creation using the real service route contract server.
- Re-run existing `ServiceApiClient` channel route contract coverage.
- Run strict `clippy` for `kamn-sdk` and targeted SDK tests.

# Deviations
- None.

# Execution evidence
- `cargo test -p kamn-sdk --test memory_agent create_channel -- --nocapture`
- `cargo test -p kamn-sdk --test live_transport_agent -- --nocapture`
- `cargo test -p kamn-sdk --test service_api_client functional_service_api_client_executes_signed_http_route_contracts -- --exact --nocapture`
- `cargo test -p kamn-sdk -- --nocapture`
- `cargo clippy -p kamn-sdk --tests -- -D warnings`
