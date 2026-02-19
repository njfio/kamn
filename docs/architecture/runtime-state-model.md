# Runtime State Model Invariant Helpers

This document records the shared helper layer used by deterministic property
runner suites for runtime-adjacent lifecycle state models.

## Helper Library

- Source: `crates/kamn-core/tests/property_invariant_helpers.rs`
- Purpose:
  - centralize deterministic proptest configuration,
  - standardize seed override behavior,
  - encode shared transition-legality projections.

## Seed Configuration Contract

- `parse_seed_override` accepts `None`, decimal, or `0x`-prefixed hex values.
- Invalid seed overrides fall back to deterministic defaults.
- `deterministic_proptest_config` uses:
  - `FileFailurePersistence::SourceParallel("proptest-regressions")`,
  - `RngAlgorithm::ChaCha`,
  - fixed `RngSeed`.

## Shared Transition Invariant APIs

- `is_legal_task_state_step` encodes allowed task lifecycle state edges.
- `expected_peer_next_state` encodes allowed peer lifecycle transition edges.

These helpers are consumed by:
- `crates/kamn-core/tests/task_escrow_proptest_invariants.rs`
- `crates/kamn-core/tests/peer_lifecycle_proptest_invariants.rs`
