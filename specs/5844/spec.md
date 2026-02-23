# Spec: Issue #5844 - Cryptographic TCP Envelope Signatures

- Issue: #5844
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
TCP relay transport in `kamn-sdk` currently uses deterministic `sig:ed25519:baseline-v1` format strings that are forgeable and do not provide cryptographic authenticity for sender/nonce/state/body fields.

## Scope
In scope:
- Replace TCP envelope signature generation/verification with cryptographic signing and verification.
- Bind signature verification to sender DID, nonce, state hash, and body.
- Carry signer public key material required for receiver-side verification in envelope/handshake payloads.
- Keep replay detection semantics and reason fields stable.
- Update TCP examples/tests to use the cryptographic signing flow.

Out of scope:
- Upstream Kolme protocol changes.
- Service API auth behavior (covered by issue #5843).

## Acceptance Criteria
- AC-1: `TcpSignedEnvelope` signatures are cryptographic and payload-bound; deterministic baseline-v1 format signatures are not accepted as valid.
- AC-2: TCP handshake validation includes cryptographic signer metadata and rejects forged handshake signatures.
- AC-3: Tampered envelope body/signature mismatches fail closed with deterministic invalid-input errors.
- AC-4: Replay detection behavior remains stable for duplicate nonce on the same route.
- AC-5: SDK examples and tests use the cryptographic TCP envelope signing contract.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | baseline-v1 deterministic signature payload | rejected as invalid signature |
| C-02 | AC-1 | Functional | signed envelope + matching public key | roundtrip parse/verify passes |
| C-03 | AC-3 | Regression | tampered body with unchanged signature | rejected with signature mismatch |
| C-04 | AC-2 | Regression | forged handshake signature with valid envelope | rejected on `handshake.signature` |
| C-05 | AC-4 | Integration | duplicate nonce across reconnect | replay conflict unchanged |
| C-06 | AC-5 | Functional | TCP sender/listener examples | compile and run with crypto signing contract |

## Test Mapping
- `cargo test -p kamn-sdk tcp_transport_adapter -- --nocapture`
- `cargo test -p kamn-sdk tcp_failover_matrix -- --nocapture`
- `cargo test -p kamn-sdk -- --nocapture`

## Success Metrics / Observable Signals
- No TCP relay path accepts deterministic baseline-v1 signatures as valid envelope auth.
- Forged handshake signatures fail closed deterministically.
- Replay-detection reason signature remains stable.
