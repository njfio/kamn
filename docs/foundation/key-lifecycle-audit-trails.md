# Key Lifecycle Tamper-Evident Audit Trails (Issue #158)

This document captures the first implementation slice for tamper-evident key lifecycle audit trails and verification checks.

## Scope Delivered
- Added deterministic audit record construction in `crates/kamn-core/src/key_lifecycle.rs`:
  - `KeyLifecycleAuditRecord` with `sequence`, `event_kind`, `event_payload`, `previous_hash`, and `record_hash`.
  - `KeyLifecycle::audit_records()` to materialize a hash-chained audit trail from lifecycle events.
  - `KeyLifecycle::verify_audit_trail()` and `KeyLifecycle::verify_audit_records(...)` for integrity validation.
  - `KeyLifecycleAuditError` typed failures for empty trails, sequence gaps, broken chain links, and hash mismatches.
- Extended integration tests in `crates/kamn-core/tests/key_lifecycle.rs` for chain construction and tamper detection.

## Tamper-Evident Rules
- The first record must reference the genesis marker `GENESIS`.
- Sequence IDs must be contiguous and start at `1`.
- Each record hash is computed from:
  - sequence
  - event kind
  - canonical event payload
  - previous hash
- Verification fails when sequence continuity, chain links, or record hashes are inconsistent.

## Limitations (First Slice)
- Hashing currently uses a deterministic non-cryptographic fingerprint for low-dependency bootstrap compatibility.
- A future slice can replace the digest with SHA-256/HMAC signing while preserving record format and verification semantics.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test key_lifecycle
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
