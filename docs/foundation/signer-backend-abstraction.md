# Pluggable Signer Backend Abstraction (Issue #160)

This document captures the first implementation slice for signer backend abstraction with deterministic secure-to-local fallback behavior.

## Scope Delivered
- Added `crates/kamn-core/src/signer_backend.rs` with:
  - `SigningRequest` contract and validation.
  - `CanonicalSecureKeyReference::parse(...)` for deterministic provider/role key-reference normalization.
  - `SignerBackend` trait (`sign` and `verify`).
  - `SecureSignerProviderClient` interface with deterministic provider adapter mapping.
  - `LocalSignerBackend` and `SecureSignerBackend`.
  - `SignerBackendRouter` with deterministic `sign_with_secure_fallback(...)` semantics plus provider-client injection.
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
  - provider-client backend mismatches are rejected with typed `ProviderClientBackendMismatch`.
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
- baseline signatures carry explicit metadata prefix: `sig:ed25519:baseline-v1:<sender>:<nonce>:<state_hash>:<payload_len>`.
- `signature_profile_compatibility_fixtures_for_fields(...)` defines migration fixtures for signer/transaction parity:
  - `baseline-v1` fixture is accepted.
  - `legacy-unversioned`, `baseline-v0`, and `secp256k1+baseline-v1` fixtures are rejected.
- parsed signature metadata is enforced through shared profile verification (`parse_signature_profile_metadata(...)`).
- baseline signature algorithm: `ed25519`.
- baseline signature profile id: `baseline-v1`.
- non-versioned signature profile is rejected (`Regression: #404`).
- algorithm/profile drift is rejected (`Regression: #677`).
- signer and transaction compatibility fixture matrix decisions stay aligned (`Regression: #677`).

## Signer Emulator Contract Lanes
- Fast PR lane (low-cost):
  - `bash scripts/signer/run_signer_emulator_contract_lane.sh`
  - includes provider handshake matrix functional + regression checks:
    - `cargo test -p kamn-core --test signer_backend functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider`
    - `cargo test -p kamn-core --test signer_backend functional_router_uses_custom_provider_client_mapping_for_secure_provider`
    - `cargo test -p kamn-core --test signer_backend regression_provider_handshake_policy_block_rejects_without_fallback`
    - `cargo test -p kamn-core --test signer_backend regression_provider_client_backend_mismatch_is_rejected_without_fallback`
    - `cargo test -p kamn-core --test signer_backend integration_signature_profile_fixture_matrix_remains_consistent_with_transaction_guards`
- Scheduled provider-integration deep lane:
  - `bash scripts/signer/run_signer_provider_deep_lane.sh`
- Signer policy fast lane:
  - `bash scripts/signer/run_signer_policy_contract_lane.sh`
  - includes privileged-role fallback denial + handshake decision matrix checks:
    - `cargo test -p kamn-core --test signer_backend functional_privileged_roles_deny_fallback_when_provider_unavailable`
    - `cargo test -p kamn-core signer_backend::tests::router_decision_matrix_distinguishes_unavailable_vs_policy_blocked_handshakes`
    - `cargo test -p kamn-core --test signer_backend regression_provider_client_backend_mismatch_is_rejected_without_fallback`
- Contract lane guards remain required for signer provider compatibility (`Regression: #619`).
- Privileged-role fallback and handshake policy bypass attempts remain fail-closed (`Regression: #987`).

## Local Validation
Run from repository root:

```bash
bash scripts/signer/run_signer_emulator_contract_lane.sh
bash scripts/signer/run_signer_policy_contract_lane.sh
bash scripts/signer/run_signer_provider_deep_lane.sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
