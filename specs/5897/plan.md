# Plan: Issue #5897 - Cryptographic Signer Migration for Core Signer Backend

## Approach
1. Add signer key-material resolution for signer backend requests using explicit runtime key material contracts.
2. Replace deterministic baseline-v1 sign/verify path in `signer_backend.rs` with cryptographic secp256k1 sign/verify helpers.
3. Add explicit legacy compatibility gate for baseline-v1 verification (default off).
4. Update unit/integration/conformance tests in `crates/kamn-core/tests/signer_backend.rs` and `crates/kamn-core/src/signer_backend.rs`.
5. Run mutation testing for touched signer verification paths and record zero-missed evidence.

## Affected Modules
- `crates/kamn-core/src/signer_backend.rs`
- `crates/kamn-core/src/signature_profile.rs` (shared helper usage)
- `crates/kamn-core/tests/signer_backend.rs`

## Risks and Mitigations
- Risk: Existing tests/docs assume baseline-v1 deterministic signatures.
  - Mitigation: add explicit compatibility switch and update tests to assert default fail-closed behavior.
- Risk: Env-key dependency introduces flaky tests.
  - Mitigation: use deterministic per-test key fixtures guarded by scoped env lock helpers.
- Risk: Router fallback policy regression.
  - Mitigation: preserve existing provider-handshake/role-policy assertions unchanged.

## Interfaces / Contracts
- Signing/verification contract in `SignerBackend` moves from deterministic format comparison to cryptographic verification.
- Compatibility switch contract is explicit and disabled by default.
- No wire-format changes outside signer backend API signatures returned to callers.

## ADR
- Not required (security hardening within existing signer architecture and provider policy model).
