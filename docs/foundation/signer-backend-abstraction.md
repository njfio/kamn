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
- `secure-mock` backend:
  - requires key IDs with `secure:` prefix.
  - returns explicit `ProviderUnavailable` when disabled/unavailable.
- Router fallback:
  - falls back from secure to local only for `ProviderUnavailable`.
  - does not fallback on hard request errors (for example, unsupported secure key references).

## Transaction Path Integration
- `SigningRequest::for_transaction(...)` maps `BaselineTransaction` fields into signer requests.
- Signed output remains compatible with `TransactionGuards::validate_and_record(...)`.
- `baseline_signature_for_fields(...)` provides the canonical signature-profile helper consumed by both paths.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test signer_backend
cargo test -p kamn-core --test transaction_guards_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
