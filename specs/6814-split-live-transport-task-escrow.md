# 6814 Split live transport task escrow tests

## Objective
Split `crates/kamn-sdk/tests/live_transport_task_escrow.rs` into a thin root shell plus bounded concern-based modules while preserving the current live transport, task, and escrow integration coverage.

## Inputs/Outputs
- Input: the current monolithic `live_transport_task_escrow.rs` test target on `main`
- Output: a root shell that wires bounded sibling modules for the current transport/task/escrow concerns and shared support

## Boundaries/Non-goals
- Do not change production transport, task, or escrow behavior
- Do not add new dependencies
- Do not weaken or delete current assertions
- Do not alter public APIs or production error semantics

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared support drifts from the original test semantics
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [ ] `crates/kamn-sdk/tests/live_transport_task_escrow.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] Root shell wires bounded sibling modules for the current transport/task/escrow concerns and shared support
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-sdk --test live_transport_task_escrow_extraction_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-sdk --test live_transport_task_escrow -- --nocapture` passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6814-touched-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6814-split-live-transport-task-escrow.md`
- `crates/kamn-sdk/tests/live_transport_task_escrow.rs`
- `crates/kamn-sdk/tests/live_transport_task_escrow_extraction_contract.rs`
- `crates/kamn-sdk/tests/live_transport_task_escrow/**`

## Error semantics
- Tests remain fail-closed and preserve the current panic/assert behavior for unexpected transport/task/escrow drift
- Shared helpers may panic with explicit messages when fixture state or runtime setup fails
- No silent fallbacks or weakened integration assertions are introduced

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the target into bounded sibling modules plus shared support
3. Run the extraction contract target
4. Run the real `live_transport_task_escrow` target
5. Run the touched-Rust size checker against `origin/main`
