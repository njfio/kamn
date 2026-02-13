# Property Invariant Matrix

This document tracks bounded property-style invariant coverage for high-risk lifecycle state machines.

## Scope

Covered in `crates/kamn-core/tests/lifecycle_property_shrinking.rs`:

- task lifecycle transitions (`TaskLifecycle`)
- peer lifecycle transitions (`PeerLifecycle`)
- escrow lifecycle transitions (`EscrowLifecycle`)

## Contract Markers

- `property_matrix_schema=kamn.lifecycle.property.v1`
- `seed_model=deterministic_xorshift64`
- `sequence_shrinker=minimal_failing_prefix`
- `ci_budget_mode=bounded`
- `counterexample_context=seed_and_shrunk_prefix`

## Seed and Budget Profile

- deterministic seed set size: `16`
- max generated sequence length per machine: `24`
- bounded PR-lane budget target: `< 250ms`

## Invariant Set

- Task lifecycle:
  - rejected transitions must not mutate state/history
  - accepted transitions must map to legal state edges
- Peer lifecycle:
  - invalid transition attempts must retain previous state
  - accepted transitions must match deterministic transition map
- Escrow lifecycle:
  - amount conservation (`released + refunded + remaining == total`) always holds
  - rejected transitions must not mutate status or amounts
  - rejection reason codes remain from stable escrow error taxonomy

## Evidence Commands

- Property lane:
  - `cargo test -p kamn-core --test lifecycle_property_shrinking`
- Existing lifecycle evidence matrix:
  - `cargo test -p kamn-core --test lifecycle_evidence_property_matrix`
- Runtime lifecycle integration lane:
  - `cargo test -p kamn-core --lib runtime::tests::`

## Regression

- Regression: #2692
