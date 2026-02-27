# Plan: Issue #6017

## Approach
1. Add fixture constructors for valid canonical envelope, ciphertext metadata, and record input.
2. Add happy-path ledger append + hash-chain verification test.
3. Add failure-path tests for duplicate message id and tamper detection via unchecked mutation helper.
4. Add conformance matrix decision tests for stable vs drift detection.
5. Run targeted `kamn-core` tests scoped to `data_layer_m0`.

## Affected Modules
- `crates/kamn-core/src/data_layer_m0.rs`

## Risks / Mitigations
- Risk: brittle fixtures due strict envelope validation rules.
  Mitigation: use canonical envelope fields aligned with existing envelope contract constants.
- Risk: false-positive tamper test if chain expectations are mis-modeled.
  Mitigation: assert specific `InvalidHashChainLink` position and values.

## Interfaces / Contracts
- No API behavior changes.
- Test-only coverage addition.
