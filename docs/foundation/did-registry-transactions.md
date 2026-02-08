# DID Registry Transactions (Issue #110)

This document defines the first implementation slice for DID
register/resolve/update/revoke transaction behavior.

## Behavior
- `register(did, document)` stores a DID document for a new DID.
- Registering an already active DID returns `DidRegistryError::AlreadyRegistered`.
- Registering a revoked DID returns `DidRegistryError::Revoked`.
- `resolve(did)` returns the active DID document.
- Resolving a revoked DID returns `DidRegistryError::Revoked`.
- Resolving an unknown DID returns `DidRegistryError::NotFound`.
- `update(did, document)` replaces the active DID document for an existing DID.
- Updating a revoked DID returns `DidRegistryError::Revoked`.
- Updating an unknown DID returns `DidRegistryError::NotFound`.
- `revoke(did)` transitions an active DID to revoked.
- Re-registering a revoked DID is rejected (`DidRegistryError::Revoked`) as a regression guard.

## Validation Rules
- The DID document `id` must match the target `did`.
- Document mismatch is rejected with `DidRegistryError::DocumentDidMismatch`.

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test did_registry_transactions
cargo test -p kamn-core
```

## Notes
This slice is intentionally in-memory and deterministic so transaction logic can be
validated quickly with low CI cost before introducing persistent storage.
