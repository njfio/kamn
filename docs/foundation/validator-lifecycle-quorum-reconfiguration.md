# Validator Lifecycle and Quorum Reconfiguration (Issues #194 / #195 / #523)

This document captures the first implementation slice for validator onboarding, offboarding, and quorum updates.

## Scope Delivered
- Added `crates/kamn-core/src/validator_lifecycle.rs` with:
  - `ValidatorLifecycleManager` for:
    - `onboard_validator(...)`
    - `offboard_validator(...)`
    - `reconfigure_quorum(...)`
    - `rollback_last_transition(...)`
  - transition proof model:
    - `ValidatorTransitionProof`
  - transition audit surfaces:
    - `ValidatorTransitionRecord`
    - `ValidatorTransitionKind`
    - `transition_history()`
  - validator set snapshot model:
    - `ValidatorSetSnapshot`
  - typed errors via `ValidatorLifecycleError`.
- Added integration and regression tests in `crates/kamn-core/tests/validator_lifecycle.rs`.

## Transition Proof and Validation Rules
- Transition proofs require:
  - non-empty `proposal_id`.
  - non-empty `proof_hash`.
  - non-empty approver DID list.
  - valid DID format for each approver.
- Proof approvals must satisfy the current quorum threshold.
- Duplicate validator DIDs are rejected.
- Transition proof fingerprint (`proposal_id` + `proof_hash`) is one-time-use and replay attempts are rejected.
- Onboarding proof approver sets cannot include the candidate validator DID (self-approval rejection).
- Regression guards:
  - transition proof replay is rejected (`Regression: #523`).
  - onboarding self-approval is rejected (`Regression: #523`).

## Quorum Safety and Rollback Rules
- Quorum threshold must be in `1..=validator_count`.
- Offboarding is blocked when resulting validator count would fall below current quorum threshold.
- Quorum reconfiguration is validated against current validator count.
- Rollback restores the previous snapshot and appends a rollback transition record.

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test validator_lifecycle --test validator_lifecycle_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
