# Agent Key Hierarchy (Issue #122)

This document defines the first implementation slice for agent key hierarchy
interfaces across identity, signing, agreement, and ephemeral session keys.

## Core Types
- `KeyRole`:
  - `Identity`
  - `Signing`
  - `Agreement`
- `AgentKeyHierarchy`:
  - current role-bound key IDs
  - ephemeral session key registry
- `EphemeralSessionKey`:
  - `key_id`
  - `expires_at_secs`

## Supported Operations
- `new(identity, signing, agreement)`
  - validates non-empty key IDs
  - enforces distinct long-term role keys
- `current_key(role)`
  - returns current key ID for identity/signing/agreement role
- `rotate_signing_key(key_id)` / `rotate_agreement_key(key_id)`
  - updates role binding while preserving long-term key separation
- `register_ephemeral(session_id, key_id, expires_at_secs)`
  - registers session-scoped ephemeral key material
- `ephemeral_key(session_id)`
  - retrieves session key details
- `retire_ephemeral(session_id)`
  - removes ephemeral key binding for a session

## Validation Rules
- Empty key IDs and empty session IDs are rejected.
- Long-term key IDs cannot be reused across identity/signing/agreement roles.
- Ephemeral expiry must be positive (`> 0`).
- Duplicate ephemeral session registration is rejected.
- Retired/missing session lookups return `SessionNotFound`.

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test agent_key_hierarchy
cargo test -p kamn-core
```

## Notes
This initial slice provides deterministic, dependency-free key hierarchy
interfaces suitable for later wiring into cryptographic material management.
