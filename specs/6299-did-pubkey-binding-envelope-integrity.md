# Spec: Issue #6299 - DID/Public-Key binding in envelope integrity verification

## Objective

Enforce sender DID to signer public-key binding during envelope integrity verification so signed
envelopes cannot claim a sender DID unrelated to the key that produced the signature.

## Inputs/Outputs

- Inputs:
  - `from` agent DID string on signed envelopes.
  - `signer_public_key` compressed secp256k1 hex on signed envelopes.
  - Existing signature payload fields (`nonce`, `state_hash`, `body`, `signature`).
- Outputs:
  - `Ok(())` only when signature verification passes and `from` DID key-binding fingerprint matches
    `signer_public_key`.
  - Deterministic `InvalidInput` errors for missing/mismatched DID key binding.

## Boundaries/Non-goals

- In scope:
  - `kamn-sdk` envelope verification path (`TcpSignedEnvelope::verify_integrity`).
  - `kamn-agent-lib` envelope verification path (`CanonicalMessageEnvelope::verify_integrity`).
  - Identity/envelope call sites that must emit DID values with key-binding markers for legitimate
    success paths.
  - Integration tests validating accept/reject behavior through real entrypoints.
- Out of scope:
  - DID format grammar changes.
  - Fingerprint algorithm changes.
  - New signing algorithms or key types.
  - Backward-compatible fallback for unbound sender DIDs.

## Failure Modes

- FM-1: envelope verification still accepts sender DIDs with no key-binding marker.
- FM-2: envelope verification accepts sender DIDs whose binding fingerprint does not match
  `signer_public_key`.
- FM-3: legitimate envelopes fail because identity/envelope builders still emit unbound sender DIDs.
- FM-4: error mapping becomes nondeterministic across SDK and agent-lib.

## Acceptance Criteria

- AC-1: `TcpSignedEnvelope::verify_integrity()` rejects envelopes when `from` DID has no
  key-binding fingerprint.
- AC-2: `TcpSignedEnvelope::verify_integrity()` rejects envelopes when DID fingerprint mismatches
  `signer_public_key`.
- AC-3: `TcpSignedEnvelope::verify_integrity()` accepts envelopes when DID/public-key binding and
  signature both verify.
- AC-4: `CanonicalMessageEnvelope::verify_integrity()` returns deterministic `AgentLibError` for
  missing/mismatched DID binding.
- AC-5: integration tests in `kamn-sdk` and `kamn-agent-lib` exercise both failure paths and at
  least one passing path with bound sender DIDs.

## Files To Touch

- `crates/kamn-sdk/src/tcp.rs`
- `crates/kamn-agent-lib/src/envelope.rs`
- `crates/kamn-agent-lib/src/identity.rs`
- `crates/kamn-sdk/tests/tcp_transport_adapter.rs`
- `crates/kamn-sdk/tests/tcp_failover_matrix.rs`
- `crates/kamn-agent-lib/tests/envelope_construction.rs`
- `specs/6299-did-pubkey-binding-envelope-integrity.md`

## Error Semantics

- SDK path (`TcpSignedEnvelope::verify_integrity`):
  - Missing DID key-binding fingerprint returns:
    - `SdkError::InvalidInput { field: "from", reason: "must include key-binding fingerprint matching signer_public_key" }`
  - Mismatched DID/public-key binding returns the same deterministic `from` field failure marker.
  - Invalid signer public key hex remains:
    - `SdkError::InvalidInput { field: "signer_public_key", reason: "must be valid compressed secp256k1 public key hex" }`
- Agent-lib path:
  - Maps SDK DID-binding failure to:
    - `AgentLibError::InvalidInput { field: "from", reason: "must include key-binding fingerprint matching signer_public_key" }`
- No silent fallback; validation fails closed.

## Test Plan

- RED:
  - Add `kamn-sdk` tests that currently pass but should fail for missing/mismatched DID key
    binding.
  - Add `kamn-agent-lib` tests for deterministic missing/mismatched DID binding error semantics.
- GREEN:
  - Add DID/public-key binding enforcement in both envelope verify paths.
  - Update identity/envelope builders and test fixtures to produce bound sender DIDs.
- REFACTOR:
  - Centralize local mapping helpers for DID-binding validation errors.
  - Keep function/file size constraints satisfied.
- Verification:
  - `cargo fmt --all --check`
  - `cargo clippy -p kamn-sdk -p kamn-agent-lib --tests -- -D warnings`
  - `cargo test -p kamn-sdk -p kamn-agent-lib`
