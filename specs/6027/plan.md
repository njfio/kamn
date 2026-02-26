# Plan: Issue #6027

## Approach
1. Add a compact test fixture helper for deterministic M5 record registration in plaintext mode.
2. Write RED tests for:
   - append/hash-chain integrity happy path,
   - tampered hash-chain fail-closed,
   - owner-encrypted privacy restrictions,
   - recall drift stable/degraded decisions.
3. Keep production code unchanged unless tests expose deterministic contract bugs.
4. Run targeted `kamn-core` M5 tests and verify all conformance cases map cleanly.

## Affected Modules
- `crates/kamn-core/src/data_layer_m5_vector_integration.rs`

## Risks / Mitigations
- Risk: nondeterministic float ordering in semantic ranking.
  Mitigation: use deterministic vectors and explicit baseline ordering in tests.
- Risk: accidentally overfitting tests to internal implementation details.
  Mitigation: assert contract-level behavior (error variants/reason codes/results), not incidental internals.

## Interfaces / Contracts
- No public API changes.
- Test-only additions validating existing contracts.
