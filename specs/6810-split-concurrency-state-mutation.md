# 6810 Split concurrency state mutation tests

## Objective
Split `crates/kamn-core/tests/concurrency_state_mutation.rs` into a thin root shell plus bounded concern-based modules while preserving the existing concurrency contract coverage and performance assertions.

## Inputs/Outputs
- Input: the current monolithic `concurrency_state_mutation.rs` test target on `main`
- Output: a root shell that wires bounded sibling modules for shared concurrency support, task race coverage, lifecycle and escrow replay coverage, and performance/deep-lane assertions

## Boundaries/Non-goals
- Do not change production concurrency or escrow behavior
- Do not add new dependencies
- Do not weaken or delete current assertions, budgets, or ignored deep-lane coverage
- Do not alter public APIs or error semantics in production code

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared race helpers drift from the original concurrency semantics
- Replay summaries or error-code assertions lose determinism after extraction
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [ ] `crates/kamn-core/tests/concurrency_state_mutation.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] Root shell wires bounded sibling modules for task race coverage, lifecycle/escrow replay coverage, performance lanes, and shared support
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-core --test concurrency_state_mutation_extraction_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test concurrency_state_mutation -- --nocapture` passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6810-touched-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6810-split-concurrency-state-mutation.md`
- `crates/kamn-core/tests/concurrency_state_mutation.rs`
- `crates/kamn-core/tests/concurrency_state_mutation_extraction_contract.rs`
- `crates/kamn-core/tests/concurrency_state_mutation/**`

## Error semantics
- Tests remain fail-closed and preserve the current panic/assert behavior for unexpected race outcomes
- Shared helpers may panic with explicit messages when a thread join or state lookup fails
- No silent fallbacks or weakened replay assertions are introduced

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the target into bounded sibling modules plus shared support
3. Run the extraction contract target
4. Run the real `concurrency_state_mutation` target
5. Run the touched-Rust size checker against `origin/main`
