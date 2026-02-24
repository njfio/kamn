# Spec: Issue #5897 - Replace Baseline Pseudo-Signatures with Cryptographic Signatures in Core Signer Path

- Issue: #5897
- Status: Implemented
- Type: task
- Priority: P0
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
The core signer backend currently emits and verifies deterministic `sig:ed25519:baseline-v1:...` format strings. This is not cryptographic signing and allows signature forgery without private-key possession.

## Scope
In scope:
- Replace pseudo-signature generation and verification in `crates/kamn-core/src/signer_backend.rs` with cryptographic signing/verification.
- Bind signature verification to the full canonical payload (sender, nonce, state hash, full payload bytes).
- Make legacy baseline-v1 acceptance opt-in via explicit compatibility mode (default fail-closed).
- Update signer backend tests to prove tamper/wrong-key rejection and valid-key acceptance.

Out of scope:
- External KMS production rollout details.
- Full transport-wide signature migration in every crate.

## Acceptance Criteria
### AC-1 Cryptographic signer output in default mode
Given a valid `SigningRequest` and configured key material,
When signing through `SignerBackendRouter` / `SecureSignerBackend`,
Then the signature output is cryptographic (`secp256k1:baseline-v2`) and not the deterministic baseline-v1 format.

### AC-2 Signature binds full payload bytes
Given a signed request,
When any of sender / nonce / state hash / payload bytes are changed,
Then verification fails.

### AC-3 Legacy baseline-v1 is fail-closed by default
Given a baseline-v1 deterministic signature,
When verification runs in default mode,
Then verification rejects the signature.

### AC-4 Legacy baseline-v1 can be explicitly enabled for compatibility
Given a baseline-v1 deterministic signature and compatibility mode enabled,
When verification runs,
Then baseline-v1 can be accepted only through the explicit compatibility switch.

### AC-5 Mutation gate for touched signer verification paths
Given the branch diff for #5897,
When `cargo mutants` runs on touched signer verification code,
Then there are zero missed mutants.

## Conformance Cases
- C-01 (Functional, AC-1): signer backend produces `sig:secp256k1:baseline-v2:<recovery_id>:<signature_hex>`.
- C-02 (Conformance, AC-2): tampered payload/nonce/state_hash/signer-key cases fail verification.
- C-03 (Conformance, AC-3): baseline-v1 signatures fail verification in default mode.
- C-04 (Conformance, AC-4): baseline-v1 signatures pass only when compatibility env switch is enabled.
- C-05 (Mutation, AC-5): in-diff mutation run shows `0 missed` for touched signer path.

## Success Metrics / Observable Signals
- No default signer-path acceptance of deterministic baseline-v1 strings.
- Signer verification requires valid cryptographic material.
- Existing role/handshake/fallback policy tests remain green.
- Mutation evidence reports zero missed mutants in touched signer paths.
