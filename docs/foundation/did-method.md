# DID Method and Canonical DID Document Schema (Issues #108, #109)

This document captures the first implementation slice for KAMN DID method handling and canonical DID document construction.

## Scope Delivered
- Added `crates/kamn-core/src/did.rs` with:
  - `AgentDid` parser for `kamn:did:agent:<method-specific-id>`
  - `AgentDidMetadata` model
  - `DidDocument`, `DidVerificationMethod`, and `DidService` models
  - `canonical_did_document(...)` builder
  - typed parsing/build errors (`AgentDidError`, `DidDocumentError`)
- Added integration tests in `crates/kamn-core/tests/did_method.rs`.

## DID Validation Rules
- DID must start with `kamn:did:agent:`.
- Method-specific ID must be non-empty.
- Method-specific ID must use lowercase alphanumeric, `_`, or `-`.

## Canonical DID Document Rules
- Contexts:
  - `https://www.w3.org/ns/did/v1.1`
  - `https://kamn.network/context/v1`
- Controller equals DID ID.
- Default verification key id: `<did>#keys-1`.
- Default service id: `<did>#messaging`.
- Default service endpoint: `kamn://messaging/<method-specific-id>`.
- Public key and capability entries must be non-empty.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test did_method
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
