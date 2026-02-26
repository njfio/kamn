# Plan: Issue #6019

## Approach
1. Add fixture helpers for deterministic valid M1 leaves and anchor results.
2. Add happy-path test for assemble + inclusion proof verify/evaluate.
3. Add error-path tests for invalid content hash and tampered proof invalidation.
4. Add failure-matrix stable/drift decision tests.
5. Run scoped `kamn-core` tests for `data_layer_m1`.

## Affected Modules
- `crates/kamn-core/src/data_layer_m1.rs`

## Risks / Mitigations
- Risk: brittle assertions against derived digest values.
  Mitigation: assert deterministic ordering/decision semantics and verification outcomes, not specific digest literals.
- Risk: false-negative tamper test if wrong field is mutated.
  Mitigation: mutate merkle root directly and assert invalid proof decision marker.

## Interfaces / Contracts
- No public API behavior changes.
- Test-only coverage addition.
