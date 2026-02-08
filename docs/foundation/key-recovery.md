# Key Compromise Revocation and Recovery Workflow (Issues #148, #149)

This document captures the first implementation slice for deterministic key compromise containment and recovery.

## Scope Delivered
- Added `crates/kamn-core/src/key_recovery.rs` with:
  - `RecoveryState` (`Active`, `Compromised`, `Revoked`, `RecoveryPending`)
  - `KeyRecoveryManager` compromise/revocation/recovery flow
  - authorization + quorum checks for recovery approval
  - replay-safe nonce handling for recovery proposals
  - compromised key reuse blocking
- Added integration tests in `crates/kamn-core/tests/key_recovery.rs`.

## Recovery Flow
1. `declare_compromised(reason)` from `Active`.
2. `emergency_revoke()` from `Compromised`.
3. `propose_recovery(candidate_key, proposer, nonce)` from `Revoked`.
4. `approve_recovery(approver)` by authorized approvers.
5. `finalize_recovery()` once approval quorum is met.

## Safety Controls
- Reject unauthorized approvers.
- Reject duplicate approvals from the same approver.
- Reject replayed proposal nonces.
- Reject reuse of compromised key IDs.
- Reject finalize when approval quorum is not met.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test key_recovery
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
