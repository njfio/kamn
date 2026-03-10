# 6812 Split public API surface policy tests

## Objective
Split `crates/kamn-core/tests/public_api_surface_policy.rs` into a thin root shell plus bounded concern-based modules while preserving the existing public-API policy contract coverage.

## Inputs/Outputs
- Input: the current monolithic `public_api_surface_policy.rs` test target on `main`
- Output: a root shell that wires bounded sibling modules for the current public-API policy concerns and shared support

## Boundaries/Non-goals
- Do not change production public-API policy behavior
- Do not add new dependencies
- Do not weaken or delete current assertions
- Do not alter public APIs or error semantics in production code

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared support drifts from the original policy assertions
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [ ] `crates/kamn-core/tests/public_api_surface_policy.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] Root shell wires bounded sibling modules for the current policy concerns and shared support
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-core --test public_api_surface_policy_extraction_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test public_api_surface_policy -- --nocapture` passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6812-touched-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6812-split-public-api-surface-policy.md`
- `crates/kamn-core/tests/public_api_surface_policy.rs`
- `crates/kamn-core/tests/public_api_surface_policy_extraction_contract.rs`
- `crates/kamn-core/tests/public_api_surface_policy/**`

## Error semantics
- Tests remain fail-closed and preserve the current panic/assert behavior for unexpected policy drift
- Shared helpers may panic with explicit messages when fixture files cannot be read
- No silent fallbacks or weakened policy assertions are introduced

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the target into bounded sibling modules plus shared support
3. Run the extraction contract target
4. Run the real `public_api_surface_policy` target
5. Run the touched-Rust size checker against `origin/main`

## Results
- [x] `crates/kamn-core/tests/public_api_surface_policy.rs` is reduced to a thin root shell at or below 180 LOC
- [x] Root shell wires bounded sibling modules for the current policy concerns and shared support
- [x] All extracted files touched by the split remain at or below 200 LOC
- [x] `cargo test -p kamn-core --test public_api_surface_policy_extraction_contract -- --nocapture` passes
- [x] `cargo test -p kamn-core --test public_api_surface_policy -- --nocapture` passes
- [x] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6812-touched-size.json` returns `policy_decision=GO`

## Phase 6 Evidence
- Root shell size: `10` LOC
- Top-level extracted modules: `17`, `16`, `1`, `1`, `23` LOC
- Nested support modules: `63`, `26`, `8`, `50`, `47`, `96`, `19`, `90`, `84`, `81` LOC
- Real `public_api_surface_policy` target remains wired through the root shell and preserves the existing schema and threshold-policy assertions

## Command Evidence
- `cargo test -p kamn-core --test public_api_surface_policy_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test public_api_surface_policy -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6812-touched-size.json`

## Deviations
- The split keeps two reserved shell modules (`task_artifact_policy_shell.rs`, `response_envelope_policy_shell.rs`) as explicit placeholders for future tranche extraction without weakening current assertions.
