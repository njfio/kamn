# 6582-harden-sdk-tcp-handshake-compares

## Objective
Replace direct equality in `TcpSignedEnvelopeHandshake::verify_matches_envelope()` with constant-time comparison for `signer_public_key` and `signature` so the SDK TCP handshake does not rely on plain string comparison at the authentication boundary.

## Inputs/Outputs
- Input: `TcpSignedEnvelopeHandshake` and `TcpSignedEnvelope`
- Output success: `Ok(())` when all handshake fields match the envelope
- Output failure:
  - `SdkError::InvalidInput { field: "handshake.signer_public_key", reason: "does not match envelope signer public key" }`
  - `SdkError::InvalidInput { field: "handshake.signature", reason: "does not match envelope signature" }`
  - existing mismatch errors for `from`, `to`, and `nonce`

## Boundaries/Non-goals
- No handshake wire-format changes
- No repo-wide constant-time sweep
- No new dependency if the helper can stay self-contained in `kamn-sdk`
- No CI/workflow/docs policy changes

## Failure modes
- Matching handshake and envelope stop verifying successfully
- Signer public key mismatch changes typed error field/reason
- Signature mismatch changes typed error field/reason
- Regression to direct string equality for signer public key or signature

## Acceptance criteria
- [x] `TcpSignedEnvelopeHandshake::verify_matches_envelope()` uses constant-time comparison for `signer_public_key` and `signature`
- [x] Matching handshake/envelope pairs still verify successfully
- [x] Signer public key mismatch still returns `SdkError::InvalidInput { field: "handshake.signer_public_key", reason: "does not match envelope signer public key" }`
- [x] Signature mismatch still returns `SdkError::InvalidInput { field: "handshake.signature", reason: "does not match envelope signature" }`
- [x] A regression test fails if implementation reverts to direct equality for those two fields

## Files to touch
- `crates/kamn-sdk/src/tcp.rs`
- `crates/kamn-sdk/tests/tcp_handshake_constant_time.rs`
- `crates/kamn-sdk/tests/support/tcp_handshake_constant_time_support.rs`
- `fixtures/ci/test_file_size_policy_baseline.env`
- `specs/6582-harden-sdk-tcp-handshake-compares.md`

## Error semantics
- Preserve current typed mismatch errors exactly
- Do not log inside handshake verification helpers
- Fail closed on any mismatch

## Test plan
- Add a dedicated TCP handshake regression test file so the existing oversized transport tests do not grow further
- Add a source-contract regression that requires constant-time comparison in `verify_matches_envelope()`
- Add runtime tests for matching handshakes and for signer/signature mismatches preserving existing typed errors
- Run `cargo test -p kamn-sdk --test tcp_handshake_constant_time -- --nocapture`
- Run `cargo clippy -p kamn-sdk --tests -- -D warnings`

## Integration verification
- The real integrated path remains `TcpTransportAdapter::listen_once()` via handshake verification inside `crates/kamn-sdk/src/tcp.rs`
- Verified through the dedicated TCP handshake test target without mock-only production bypasses
- Verified workspace inventory parity after adding the dedicated test target

## Verification actuals
- Red: `cargo test -p kamn-sdk --test tcp_handshake_constant_time -- --nocapture`
  - `regression_requires_constant_time_tcp_handshake_compares` failed because `verify_matches_envelope()` still used direct equality
- Green: `cargo test -p kamn-sdk --test tcp_handshake_constant_time -- --nocapture`
- Green: `cargo clippy -p kamn-sdk --tests -- -D warnings`
- Green: `cargo test -p kamn-sdk -- --nocapture`
- Green: `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`

## Deviations
- Adding the dedicated TCP handshake test target increased workspace test inventory by one file, so `fixtures/ci/test_file_size_policy_baseline.env` was refreshed from `460` to `461`.
