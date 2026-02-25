# Plan: Issue #5994

## Approach
1. Extend service-api persisted snapshot schema with a `bridges` map (serde-defaulted for compatibility).
2. Add bridge lifecycle methods in `message_store.rs`:
   - submit bridge
   - forward bridge
   - query bridge
3. Wire bridge routes in `middleware_impl.rs` to call message-store methods and return fail-closed `404` for unknown bridge IDs.
4. Add restart integration test in `main_tests/service_api_endpoint_tests.rs`.
5. Re-run durable relay integration test to guard regression in existing cross-node flow.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks / Mitigations
- Risk: state snapshot schema extension could break older state files.
  Mitigation: `#[serde(default)]` for new `bridges` field.
- Risk: route handling overlap could alter legacy fallback behavior.
  Mitigation: preserve existing `payload.rs` fallback for non-live projection path, but ensure middleware short-circuits live route handling.
- Risk: anti-spam/request budget interference in long integration sequence.
  Mitigation: keep request counts bounded and vary caller DID when needed.

## Interfaces / Contracts
- Route paths and response field names remain unchanged.
- Live bridge behavior becomes durable/persistent.
- Error taxonomy remains existing deterministic reason-code set.
