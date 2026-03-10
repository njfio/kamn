# 6816 Split shell test surface ratio policy tests

## Objective
Split `crates/kamn-core/tests/shell_test_surface_ratio_policy.rs` into a thin root shell plus bounded concern-based modules while preserving the current shell-test-surface ratio policy coverage.

## Inputs/Outputs
- Input: the current monolithic `shell_test_surface_ratio_policy.rs` test target on `main`
- Output: a root shell that wires bounded sibling modules for the current ratio-policy concerns and shared support

## Boundaries/Non-goals
- Do not change production shell-test-surface ratio policy behavior
- Do not add new dependencies
- Do not weaken or delete current assertions
- Do not alter public APIs or production error semantics

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared support drifts from the original policy assertions
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [ ] `crates/kamn-core/tests/shell_test_surface_ratio_policy.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] Root shell wires bounded sibling modules for the current ratio-policy concerns and shared support
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-core --test shell_test_surface_ratio_policy_extraction_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test shell_test_surface_ratio_policy -- --nocapture` passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6816-touched-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6816-split-shell-test-surface-ratio-policy.md`
- `crates/kamn-core/tests/shell_test_surface_ratio_policy.rs`
- `crates/kamn-core/tests/shell_test_surface_ratio_policy_extraction_contract.rs`
- `crates/kamn-core/tests/shell_test_surface_ratio_policy/**`

## Error semantics
- Tests remain fail-closed and preserve the current panic/assert behavior for policy drift
- Shared helpers may panic with explicit messages when fixtures cannot be read
- No silent fallbacks or weakened policy assertions are introduced

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the target into bounded sibling modules plus shared support
3. Run the extraction contract target
4. Run the real `shell_test_surface_ratio_policy` target
5. Run the touched-Rust size checker against `origin/main`
