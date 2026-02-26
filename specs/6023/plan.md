# Plan: Issue #6023

## Approach
1. Add test fixtures for deterministic metadata records and shared blind-index tokens.
2. Add happy-path exact-match query ordering test.
3. Add regression test for duplicate registration and invalid token rejection.
4. Add determinism stable/drift decision test.
5. Run scoped `kamn-core` library tests for `data_layer_m3_blind_index_search`.

## Affected Modules
- `crates/kamn-core/src/data_layer_m3_blind_index_search.rs`

## Risks / Mitigations
- Risk: brittle assertions against generated digest strings.
  Mitigation: assert ordering and decision outputs instead of literal digest values.
- Risk: weak coverage for error path semantics.
  Mitigation: assert full typed errors for duplicate registration and invalid token queries.

## Interfaces / Contracts
- No public API behavior changes.
- Test-only coverage addition.
