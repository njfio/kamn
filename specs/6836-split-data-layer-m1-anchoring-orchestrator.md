# 6836-split-data-layer-m1-anchoring-orchestrator

## Objective
Reduce `crates/kamn-core/tests/data_layer_m1_anchoring_orchestrator.rs` from a 509 LOC monolithic test target to a thin root shell plus bounded concern modules without changing anchoring orchestrator behavior or weakening the existing contract coverage.

## Inputs/Outputs
- Input: existing `kamn-core` anchoring orchestrator contract tests in `crates/kamn-core/tests/data_layer_m1_anchoring_orchestrator.rs`
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
- Anchoring tick/follow-up assertions or reason-code checks change during extraction
- Touched-Rust size policy remains `NO-GO`

## Acceptance Criteria
- [ ] `crates/kamn-core/tests/data_layer_m1_anchoring_orchestrator.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] The root shell wires bounded sibling modules for the current anchoring orchestrator concerns
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-core --test data_layer_m1_anchoring_orchestrator -- --nocapture` passes
- [ ] A new extraction contract for the root shell passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json <path>` returns `policy_decision=GO`

## Files To Touch
- `specs/6836-split-data-layer-m1-anchoring-orchestrator.md`
- `crates/kamn-core/tests/data_layer_m1_anchoring_orchestrator.rs`
- `crates/kamn-core/tests/data_layer_m1_anchoring_orchestrator/`
- `crates/kamn-core/tests/data_layer_m1_anchoring_orchestrator_extraction_contract.rs`

## Error Semantics
- Preserve all existing `DataLayerM1AnchoringOrchestratorError` assertions unchanged
- Preserve all reason-code assertions unchanged
- Extraction contract failures must fail closed with explicit missing-file, missing-marker, or root-budget assertions
- No silent fallbacks or weakened checks

## Test Plan
1. Add an extraction contract that fails while the root file remains monolithic.
2. Split the root into bounded modules by concern:
   - support fixtures and scripted client
   - tick planning and persistence metadata
   - rejected/final receipt outcomes
   - finality observation and follow-up policy reconciliation
3. Run `cargo test -p kamn-core --test data_layer_m1_anchoring_orchestrator_extraction_contract -- --nocapture`.
4. Run `cargo test -p kamn-core --test data_layer_m1_anchoring_orchestrator -- --nocapture`.
5. Run the touched-Rust size ratchet against the issue write set and require `policy_decision=GO`.

## Phase 6 Evidence
- Root shell wiring verified through `cargo test -p kamn-core --test data_layer_m1_anchoring_orchestrator -- --nocapture`
- Extraction contract verified through `cargo test -p kamn-core --test data_layer_m1_anchoring_orchestrator_extraction_contract -- --nocapture`
- Touched-Rust policy verified through `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6836-touched-size.json`
- Result: `policy_decision=GO`
