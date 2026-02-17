# Plan — #4332

Status: Reviewed

## Approach

- Add red unit/regression tests in `crates/kamn-node/src/main_tests/runtime_tests.rs` for:
  - contradictory graceful drain/timeout metadata,
  - invalid numeric shutdown reconciliation fields,
  - checkpoint reason/counter drift.
- Keep tests deterministic by using direct classifier/validator calls and stable reason-code assertions.
- Pair with #4333 implementation in the same change set so final branch remains green.

## Affected Areas

- `crates/kamn-node/src/main_tests/runtime_tests.rs`

## Risks and Mitigations

- Risk: reason-code naming drift between tests and implementation.
  - Mitigation: define deterministic reason strings once and assert exact values.
- Risk: over-constraining shutdown metadata semantics.
  - Mitigation: only enforce invariants derivable from existing completion reason contracts.
