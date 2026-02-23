# Plan: Issue #5844

## Approach
1. Introduce cryptographic TCP envelope signing/verification in `crates/kamn-sdk/src/tcp.rs`.
2. Extend envelope and handshake payload contracts to include signer public key material.
3. Update TCP tests to use cryptographic signatures and preserve replay/forgery failure expectations.
4. Update TCP examples to provide signing key material explicitly.

## Affected Modules
- `crates/kamn-sdk/src/tcp.rs`
- `crates/kamn-sdk/src/lib.rs`
- `crates/kamn-sdk/tests/tcp_transport_adapter.rs`
- `crates/kamn-sdk/tests/tcp_failover_matrix.rs`
- `crates/kamn-sdk/examples/tcp_signed_relay_sender.rs`
- `crates/kamn-sdk/examples/tcp_signed_relay_listener.rs`

## Risks / Mitigations
- Risk: API break in public TCP signing helpers.
  - Mitigation: keep helper naming stable where possible and update examples/tests in the same change.
- Risk: handshake schema drift invalidates existing tests.
  - Mitigation: update all fixture payloads and keep deterministic error fields unchanged.
- Risk: env/key handling introduces nondeterminism in tests.
  - Mitigation: pass explicit private key test fixtures through envelope construction helpers.

## Interfaces / Contracts
- `TcpSignedEnvelope::new(...)` will require signing key input to emit cryptographic signatures.
- Envelope wire payload includes signer public key and cryptographic signature.
- Handshake profile marker moves to crypto profile and includes signer public key parity check.

## ADR
- Not required: cryptographic primitives are reused from existing `kamn-core` signature profile helpers.
