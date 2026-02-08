# Pluggable Signer Backend Abstraction (Issue #160)

This document captures the first implementation slice for signer backend abstraction with deterministic secure-to-local fallback behavior.

## Scope Delivered
- Added `crates/kamn-core/src/signer_backend.rs` with:
  - `SigningRequest` contract and validation.
  - `SignerBackend` trait (`sign` and `verify`).
  - `LocalSignerBackend` and `SecureSignerBackend`.
  - `SignerBackendRouter` with deterministic `sign_with_secure_fallback(...)` semantics.
  - `BackendSignature` and typed `SignerBackendError`.
- Added integration tests in `crates/kamn-core/tests/signer_backend.rs`.

## Backend Compatibility Rules
- `local-software` backend:
  - signs and verifies deterministic signatures for valid requests.
- secure provider adapters:
  - `secure-mock` for deterministic mock custody.
  - `secure-aws-kms-emulator` for production-style provider adapter coverage.
  - provider handshake matrix statuses:
    - `Available`
    - `Unavailable`
    - `PolicyBlocked`
  - key references:
    - legacy mock: `secure:<key-ref>`
    - explicit mock: `secure:mock:<key-ref>`
    - production-style adapter: `secure:aws-kms:<key-ref>`
    - role-scoped production keys: `secure:aws-kms:role-<operator|admin|treasury|auditor>/<key-ref>`
  - sender role prefixes map to signer roles (`admin-*`, `treasury-*`, `auditor-*`; default is `operator`).
  - key-role mismatch is rejected with typed `KeyRoleMismatch`.
  - unknown provider labels are rejected with typed `UnsupportedSecureProvider`.
  - unknown role labels are rejected with typed `UnsupportedSignerKeyRole`.
  - malformed secure key references are rejected with typed `MalformedSecureKeyReference`.
  - provider handshake policy blocks are rejected with typed `ProviderHandshakeRejected`.
  - `ProviderUnavailable` reports the provider-specific backend when the secure path is disabled.
- Router fallback:
  - falls back from secure to local only for `ProviderUnavailable` and `operator` role keys.
  - fallback is denied for privileged roles with typed `FallbackDeniedByRolePolicy`.
  - fallback is denied for handshake policy blocks (`ProviderHandshakeRejected`) (`Regression: #677`).
  - does not fallback on hard request errors (for example, unsupported secure key references).
  - verification rejects backend/provider mismatches via `SecureProviderBackendMismatch` (`Regression: #619`).

## Transaction Path Integration
- `SigningRequest::for_transaction(...)` maps `BaselineTransaction` fields into signer requests.
- Signed output remains compatible with `TransactionGuards::validate_and_record(...)`.
- `baseline_signature_for_fields(...)` provides the canonical signature-profile helper consumed by both paths.
- baseline signature profile id: `baseline-v1`.
- non-versioned signature profile is rejected (`Regression: #404`).

## Signer Emulator Contract Lanes
- Fast PR lane (low-cost):
  - `bash scripts/signer/run_signer_emulator_contract_lane.sh`
  - includes provider handshake matrix functional + regression checks:
    - `cargo test -p kamn-core --test signer_backend functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider`
    - `cargo test -p kamn-core --test signer_backend regression_provider_handshake_policy_block_rejects_without_fallback`
- Scheduled provider-integration deep lane:
  - `bash scripts/signer/run_signer_provider_deep_lane.sh`
- Contract lane guards remain required for signer provider compatibility (`Regression: #619`).

## Local Validation
Run from repository root:

```bash
bash scripts/signer/run_signer_emulator_contract_lane.sh
bash scripts/signer/run_signer_provider_deep_lane.sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
