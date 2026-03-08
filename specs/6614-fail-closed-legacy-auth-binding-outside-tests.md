# 6614-fail-closed-legacy-auth-binding-outside-tests

## Objective
Remove debug-build activation of legacy sender/signing-key binding from the service API authentication production path so authenticated requests fail closed unless they provide an explicit signer public key header. Preserve the existing internal legacy-policy seam for targeted test-only coverage.

## Inputs/Outputs
- Inputs:
  - `ServiceApiRuntimeState`
  - `ParsedRequest`
  - `ServiceApiReplayGuard`
  - request auth headers including `X-KAMN-Sender-DID`, `X-KAMN-Nonce`, `X-KAMN-Signature`, and optionally `X-KAMN-Signer-Public-Key`
- Outputs:
  - `Ok(())` for authenticated requests that satisfy the explicit signer binding contract
  - `RequestAuthFailure::Unauthorized` when the signer public key header is missing or the sender DID does not match the explicit binding contract
  - explicit success from `authorize_service_api_request_with_legacy_policy(..., true)` only in tests that opt into the legacy seam directly

## Boundaries/Non-goals
- Do not change request scope policy.
- Do not change replay-guard behavior.
- Do not change signature verification cryptography.
- Do not remove the internal legacy-policy helper used by targeted tests.
- Do not widen legacy compatibility outside direct explicit helper calls.

## Failure modes
- Missing `X-KAMN-Signer-Public-Key` on an authenticated route returns unauthorized when the public auth entrypoint is used.
- Sender DID and signer public key mismatch returns unauthorized when the public auth entrypoint is used.
- Invalid sender DID shape continues to fail at header validation.
- Explicit legacy helper use with `allow_legacy_sender_binding = true` remains the only path that may accept legacy fallback binding.

## Acceptance criteria
- [ ] `authorize_service_api_request()` never enables legacy sender binding based on `debug_assertions` or any other build-mode flag.
- [ ] Authenticated requests without `X-KAMN-Signer-Public-Key` fail closed through `authorize_service_api_request()` even when runtime fallback public key exists.
- [ ] Targeted tests can still prove legacy fallback behavior by calling `authorize_service_api_request_with_legacy_policy(..., true)` directly.
- [ ] Targeted tests can prove `authorize_service_api_request()` rejects the same legacy fallback request shape.
- [ ] Existing explicit signer-binding success cases continue to pass.

## Files to touch
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `specs/6614-fail-closed-legacy-auth-binding-outside-tests.md`

## Error semantics
- Missing explicit signer public key header from the public auth entrypoint returns `RequestAuthFailure::Unauthorized` with `REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED` and a message containing `X-KAMN-Signer-Public-Key`.
- Sender DID binding mismatch from the public auth entrypoint returns `RequestAuthFailure::Unauthorized` with `REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED`.
- No silent fallback to runtime auth public key is allowed from the public auth entrypoint.

## Test plan
- Add a red unit test proving `authorize_service_api_request()` rejects a legacy sender request that omits `X-KAMN-Signer-Public-Key` even when the runtime state carries `auth_public_key_hex`.
- Add a red unit test proving `authorize_service_api_request_with_legacy_policy(..., true)` still accepts the same request shape when explicitly opted in.
- Run targeted auth unit tests for `crates/kamn-node/src/service_api_endpoint/auth.rs`.
- Run strict clippy for `kamn-node` tests after implementation.
