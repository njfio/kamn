# Plan: Issue #6037

## Approach
1. Extend the existing module test fixtures with non-go and non-conformant reports.
2. Add RED test asserting deterministic multi-reason ordering and projection fields.
3. Keep production code unchanged unless the test reveals contract mismatch.
4. Run targeted M11 test slices and adjacent hardening-readiness tests.

## Affected Modules
- `crates/kamn-core/src/data_layer_m11_closure_evidence.rs`

## Risks / Mitigations
- Risk: Tests might overfit to incidental implementation details.
  Mitigation: Assert only documented reason-code order and exposed output fields.

## Interfaces / Contracts
- No public API changes.
- Test-only contract coverage additions.
