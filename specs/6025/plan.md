# Plan: Issue #6025

## Approach
1. Add test fixtures for draft escrows and settlement evidence inputs.
2. Add transition/visibility happy-path test.
3. Add regression test for invalid transition and tampered evidence integrity failure.
4. Add reconciliation match/mismatch decision test.
5. Run scoped `kamn-core` library tests for `data_layer_m4_escrow_integration`.

## Affected Modules
- `crates/kamn-core/src/data_layer_m4_escrow_integration.rs`

## Risks / Mitigations
- Risk: brittle assertions tied to incidental transition internals.
  Mitigation: assert explicit states, reason codes, and typed errors only.
- Risk: tamper regression may assert wrong failure position.
  Mitigation: mutate sequence `1` hash and assert deterministic `position: 0` record-hash mismatch.

## Interfaces / Contracts
- No public API behavior changes.
- Test-only coverage addition.
