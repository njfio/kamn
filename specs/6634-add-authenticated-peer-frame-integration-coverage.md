# 6634-add-authenticated-peer-frame-integration-coverage

## Objective
Add dedicated crate-level integration coverage for the public authenticated peer frame API in `runtime_peer_coordination.rs` so wire parsing, signing, authorization, recipient checks, and replay protection remain pinned outside inline module coverage.

## Inputs/Outputs
- Inputs:
  - public `AuthenticatedPeerFrame` construction, signed construction, wire encoding/decoding, and `PeerFrameAuthenticator` validation
- Outputs:
  - dedicated integration test surface at `crates/kamn-core/tests/authenticated_peer_frame_integration.rs`
  - dedicated contract test at `crates/kamn-core/tests/authenticated_peer_frame_contract.rs`
  - refreshed `test_file_size_policy` baseline if the new test targets change workspace inventory counts

## Boundaries/Non-goals
- No production behavior changes in `crates/kamn-core/src/runtime_peer_coordination.rs`
- No proposal planner or runtime wiring coverage in this issue
- No CI or workflow changes
- No visibility changes for internal helpers solely to support tests

## Failure modes
- dedicated authenticated-peer-frame integration surface missing entirely
- valid signed frame round-trip or inbound validation regresses
- invalid sender DID, recipient DID, local peer DID, nonce, payload, or signature stop failing closed
- malformed wire payload stops failing closed
- unauthorized sender, wrong recipient, or replay nonce stop failing closed
- workspace `test_file_size_policy` inventory drifts after adding new test targets

## Acceptance criteria (testable booleans)
- [ ] `authenticated_peer_frame_contract` fails when the dedicated integration surface or its marker cases disappear
- [ ] integration coverage asserts a valid signed frame round-trips through wire format and validates inbound successfully
- [ ] integration coverage asserts malformed constructor or wire inputs fail closed with deterministic public error variants and reason markers
- [ ] integration coverage asserts unauthorized sender, wrong recipient, and replay nonce fail closed through `PeerFrameAuthenticator`
- [ ] `cargo test -p kamn-core --test authenticated_peer_frame_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test authenticated_peer_frame_integration -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` passes

## Files to touch
- `specs/6634-add-authenticated-peer-frame-integration-coverage.md`
- `crates/kamn-core/tests/authenticated_peer_frame_integration.rs`
- `crates/kamn-core/tests/authenticated_peer_frame_contract.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` if needed

## Error semantics
- Tests assert the existing public fail-closed behavior only
- Invalid constructor, wire, and inbound-authentication paths must preserve current public enum variants and deterministic reason-code markers where exposed
- No new production error types or translation layers are introduced

## Test plan
1. Add a contract test referencing `authenticated_peer_frame_integration.rs` before that file exists so the red phase is a real missing-surface failure.
2. Add a dedicated integration surface covering one valid signed round-trip and the public fail-closed constructor, wire, recipient, authorization, and replay paths.
3. Run targeted contract and integration tests.
4. Run `test_file_size_policy` and refresh its baseline only if the new test targets change inventory counts.

## Deviations
- The workspace `test_file_size_policy` inventory changed from `485` to `487`, so `fixtures/ci/test_file_size_policy_baseline.env` was refreshed during integration.

## Phase 6 Evidence
- `cargo test -p kamn-core --test authenticated_peer_frame_contract -- --nocapture`
- `cargo test -p kamn-core --test authenticated_peer_frame_integration -- --nocapture`
- `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`
- `cargo clippy -p kamn-core --tests -- -D warnings`
