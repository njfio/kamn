# Key Lifecycle and Rotation Protocol (Issues #146, #147)

This document captures the first implementation slice for deterministic key lifecycle and rotation transitions.
For tamper-evident audit trail verification controls, see `docs/foundation/key-lifecycle-audit-trails.md`.

## Scope Delivered
- Added `crates/kamn-core/src/key_lifecycle.rs` with:
  - `KeyLifecycleState` (`Active`, `Rotating`, `Revoked`)
  - `KeyLifecycle` transition engine
  - `KeyLifecycleEvent` audit events with deterministic sequence IDs
  - `KeyLifecycleError` typed transition/validation failures
- Added integration tests in `crates/kamn-core/tests/key_lifecycle.rs`.

## Implemented Transition Rules
- `Active -> Rotating` via `initiate_rotation(next_key_id)`
- `Rotating -> Active` via `activate_rotation()`
- `Active/Rotating -> Revoked` via `revoke()`
- Reject `activate_rotation()` outside `Rotating`.
- Reject rotation when next key is empty or unchanged.
- Reject rotation once lifecycle is `Revoked`.

## Audit Events
- `RotationInitiated { sequence, from_key, to_key }`
- `RotationActivated { sequence, active_key }`
- `KeyRevoked { sequence, key_id }`

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test key_lifecycle
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
