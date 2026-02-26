# Plan: Issue #6021

## Approach
1. Add test fixtures for deterministic M2 message scopes and access-audit inputs.
2. Add happy-path test for append-only audit chain assembly and verification.
3. Add regression test for tampered lineage detection via `replace_record_hash_unchecked`.
4. Add matrix-decision tests for stable and drift outcomes.
5. Run scoped `kamn-core` tests for `data_layer_m2_gateway_access`.

## Affected Modules
- `crates/kamn-core/src/data_layer_m2_gateway_access.rs`

## Risks / Mitigations
- Risk: brittle assertions on digest values.
  Mitigation: assert sequencing, chain linkage, and decision markers instead of hardcoding full digests.
- Risk: false-positive tamper test if wrong record is mutated.
  Mitigation: mutate record `sequence=1` and assert deterministic position/reason markers on verification failure.

## Interfaces / Contracts
- No public API behavior changes.
- Test-only coverage addition.
