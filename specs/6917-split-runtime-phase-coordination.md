# 6917-split-runtime-phase-coordination

## Objective
Split `crates/kamn-core/src/runtime_phase_coordination.rs` into bounded concern-based modules while preserving runtime phase coordination behavior, public surface, and existing tests.

## Inputs/Outputs
- Input: current `crates/kamn-core/src/runtime_phase_coordination.rs` production source and its existing tests.
- Output: a thin root shell delegating to bounded sibling modules for extracted runtime phase coordination concerns.
- Output: a hard-fail extraction contract that enforces the root shell budget and extracted module layout.

## Boundaries/Non-goals
- Do not change runtime phase coordination behavior or public API semantics.
- Do not add dependencies.
- Do not weaken or delete tests to make the split pass.
- Do not move unrelated runtime modules in this issue.

## Failure modes
- The root file remains oversized while the extraction contract passes.
- Extracted modules drift behavior or public surface.
- Tests compile but no real runtime path still exercises the split code.
- Any touched file or function fails the touched-Rust size policy.

## Acceptance criteria
- [ ] `crates/kamn-core/src/runtime_phase_coordination.rs` becomes a thin root shell under the active file-size policy.
- [ ] Bounded sibling modules exist for the extracted runtime phase coordination seams.
- [ ] Existing runtime phase coordination behavior remains unchanged after the split.
- [ ] A hard-fail extraction contract exists and passes.
- [ ] Real runtime phase coordination targets still pass after the split.
- [ ] Touched-Rust size policy returns `GO` on the final branch.

## Files to touch
- `crates/kamn-core/src/runtime_phase_coordination.rs`
- `crates/kamn-core/src/runtime_phase_coordination/`
- `crates/kamn-core/tests/runtime_phase_coordination_module_extraction_contract.rs`
- `specs/6917-split-runtime-phase-coordination.md`

## Error semantics
- Existing hard-fail runtime phase coordination behavior remains explicit and unchanged.
- No silent fallbacks or swallowed failures are introduced.
- Extracted seams preserve current error propagation and caller-visible behavior.

## Test plan
- Add a red extraction contract that fails while the root file remains oversized and the expected module layout is missing.
- Run the extraction contract target and confirm red.
- Run the real runtime phase coordination target after extraction.
- Run touched-Rust size policy on the final branch.
