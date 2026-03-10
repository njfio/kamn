# 6832-split-data-layer-m5-vector-integration

## Objective
Reduce `crates/kamn-core/tests/data_layer_m5_vector_integration.rs` from a monolithic 511 LOC test target to a thin root shell plus bounded concern modules without changing vector integration behavior or reducing contract coverage.

## Inputs/Outputs
- Input: existing `kamn-core` vector integration contract tests in `crates/kamn-core/tests/data_layer_m5_vector_integration.rs`
- Output: bounded sibling test modules and a root shell that only wires them together
- Output: extraction contract coverage enforcing the root shell budget and module layout

## Boundaries/Non-goals
- No production code changes in `kamn-core`
- No new dependencies
- No changes to public APIs, reason codes, or runtime error semantics
- No weakening, deleting, or consolidating away existing test assertions

## Failure Modes
- Root shell remains above the staged 180 LOC cap
- Any extracted touched file exceeds 200 LOC
- Module wiring drifts and test coverage becomes disconnected
- Assertions or reason-code checks change during extraction
- Touched-Rust size policy remains `NO-GO`

## Acceptance Criteria
- [ ] `crates/kamn-core/tests/data_layer_m5_vector_integration.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] The root shell wires bounded sibling modules for the current vector integration concerns
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-core --test data_layer_m5_vector_integration -- --nocapture` passes
- [ ] A new extraction contract for the root shell passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json <path>` returns `policy_decision=GO`

## Files To Touch
- `specs/6832-split-data-layer-m5-vector-integration.md`
- `crates/kamn-core/tests/data_layer_m5_vector_integration.rs`
- `crates/kamn-core/tests/data_layer_m5_vector_integration/`
- `crates/kamn-core/tests/data_layer_m5_vector_integration_extraction_contract.rs`

## Error Semantics
- Preserve all existing `DataLayerM5VectorIntegrationError` assertions unchanged
- Extraction contract failures must fail closed with explicit missing-file, missing-marker, or root-budget assertions
- No silent fallbacks or weakened checks

## Test Plan
1. Add an extraction contract that fails while the root file remains monolithic.
2. Split the root into bounded modules by concern:
   - support/input builders
   - registry append integrity and duplicate handling
   - semantic query scope/privacy behavior
   - anomaly evaluation behavior
   - recall drift behavior
   - retention projection and canonical owner scope behavior
3. Run `cargo test -p kamn-core --test data_layer_m5_vector_integration_extraction_contract -- --nocapture`.
4. Run `cargo test -p kamn-core --test data_layer_m5_vector_integration -- --nocapture`.
5. Run the touched-Rust size ratchet against the issue write set and require `policy_decision=GO`.
