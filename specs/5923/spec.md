# Spec: Issue #5923 - Task: Replace deterministic agent-lib auth signatures with cryptographic signatures

- Issue: #5923
- Status: Implemented
- Type: task
- Priority: P0
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5916

## Problem Statement
Auth currently provides zero authenticity because signature and verification are deterministic string recomputation.

## Scope
In scope:
- Replace `kamn-agent-lib` auth-header signing path with cryptographic service-auth signatures.
- Add deterministic forgery/tamper regression coverage for `KamnAuthHeaders::build`.
- Add SDK helper for signing service-auth signatures directly from a supplied private key + state hash.

Out of scope:
- Agent-lib API redesign unrelated to auth correctness.
- Message envelope signature model migration (separate concern from request-auth headers).

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Forged deterministic signatures no longer validate.
- AC-2: Auth signing requires real private-key material and produces verifiable signatures.
- AC-3: Regression tests cover replay/tamper/forgery negative paths.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): `spec_c03_auth_roundtrip_forged_deterministic_signature_is_rejected` asserts forged deterministic signatures do not match generated auth signatures.
- C-02 (Functional, AC-2): `spec_c01_auth_roundtrip_signature_matches_service_crypto_contract` and `spec_c02_auth_roundtrip_rejects_non_private_key_signing_material` verify signatures align with cryptographic service contract and fail closed on non-key input.
- C-03 (Regression, AC-3): `spec_c04_auth_roundtrip_tampered_same_length_payload_changes_signature` verifies same-length payload tampering changes signatures (closing deterministic length-only vulnerability).
- C-04 (Verify, AC-4): `cargo test -p kamn-agent-lib --test auth_roundtrip -- --nocapture`, `cargo test -p kamn-agent-lib -- --nocapture`, `cargo test -p kamn-sdk -- --nocapture`, `cargo fmt --check`, and strict clippy for touched crates pass.

## Success Metrics / Observable Signals
- `KamnAuthHeaders::build` no longer emits `sig:deterministic-v1:baseline-v1:*` signatures.
- Non-hex/private-key placeholder signing material fails closed.
- Same-length tampered payloads produce distinct signatures.
- Scoped agent-lib/sdk tests, format, and strict lint checks pass.


## Required Test Categories
- Unit: auth sign/verify functions
- Functional: request auth round-trip
- Integration: sdk/agent-lib authenticated request path
- Regression: deterministic signature string path removed
- Performance: auth signing latency budget

## Dependencies
- #5916
