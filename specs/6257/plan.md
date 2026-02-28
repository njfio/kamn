# Issue 6257 Plan

## Approach
1. Add spec-derived tests directly in existing crate test modules to keep scope minimal.
2. Target high-value edge/negative/determinism behaviors per crate:
   - `kamn-data-layer`: tagged hash formatting + determinism.
   - `kamn-snapshot-journal`: path helper, hex decode edge cases, invalid schema handling.
   - `kamn-bridges`: empty field validation, label normalization, pending semantics.
   - `kamn-crypto`: seed parsing, nonce/aad determinism, encoding/decoding error paths, constructor validation.
   - `kamn-types`: parse semantics + exported error surface compatibility.
3. Run focused crate test commands first, then combined multi-crate test gate.
4. Verify conformance counts with deterministic `rg` checks.

## Affected Modules
- `crates/kamn-data-layer/src/data_layer_hashing.rs`
- `crates/kamn-snapshot-journal/src/lib.rs`
- `crates/kamn-bridges/src/cross_chain_receipt.rs`
- `crates/kamn-crypto/src/direct_message_crypto.rs`
- `crates/kamn-types/src/lib.rs`

## Risks and Mitigations
- Risk: brittle tests around implementation details.
  - Mitigation: assert stable behavior contracts, not incidental internals.
- Risk: environment-sensitive crypto tests.
  - Mitigation: reuse existing seed-lock helper pattern in crate tests.

## Interface/Contract Notes
- No production API changes expected.
- Test-only additions should preserve existing external behavior.
