# 6808 Split e2e live workflow lane tests

## Objective
Split `crates/kamn-core/tests/e2e_live_workflow_lane.rs` into a thin root shell plus bounded concern-based modules while preserving the current end-to-end live workflow lane coverage.

## Inputs/Outputs
- Input: the current `e2e_live_workflow_lane.rs` monolithic test target on `main`
- Output: a root shell that wires bounded sibling modules for baseline/taxonomy helpers, live toggle and key markers, trigger/scope checks, scenario matrix checks, PR skip markers, strategy markers, and CLI smoke wrapper checks

## Boundaries/Non-goals
- Do not change production workflow or strategy documents
- Do not add new dependencies or features
- Do not weaken workflow-lane contract assertions
- Do not change public APIs

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared workflow parsing helpers drift from the original contract evaluation behavior
- Scenario matrix or strategy-marker checks lose deterministic reason-code assertions after extraction
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [ ] `crates/kamn-core/tests/e2e_live_workflow_lane.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] Root shell wires bounded sibling modules for taxonomy/baseline, live markers, trigger/scope guards, scenario/PR markers, strategy markers, and CLI smoke coverage
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-core --test e2e_live_workflow_lane_extraction_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test e2e_live_workflow_lane -- --nocapture` passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6808-touched-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6808-split-e2e-live-workflow-lane.md`
- `crates/kamn-core/tests/e2e_live_workflow_lane.rs`
- `crates/kamn-core/tests/e2e_live_workflow_lane_extraction_contract.rs`
- `crates/kamn-core/tests/e2e_live_workflow_lane/**`

## Error semantics
- Tests remain fail-closed and preserve current deterministic reason-code assertions
- Shared helpers may panic in tests with explicit messages when fixture files cannot be read
- No silent fallbacks or swallowed contract failures

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the test target into bounded sibling modules and shared support
3. Run the extraction contract target
4. Run the real `e2e_live_workflow_lane` target
5. Run the touched-Rust size checker against `origin/main`

## Results
- [x] `crates/kamn-core/tests/e2e_live_workflow_lane.rs` is reduced to a thin root shell at or below 180 LOC
- [x] Root shell wires bounded sibling modules for taxonomy/baseline, live markers, trigger/scope guards, scenario/PR markers, strategy markers, and CLI smoke coverage
- [x] All extracted files touched by the split remain at or below 200 LOC
- [x] `cargo test -p kamn-core --test e2e_live_workflow_lane_extraction_contract -- --nocapture` passes
- [x] `cargo test -p kamn-core --test e2e_live_workflow_lane -- --nocapture` passes
- [x] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6808-touched-size.json` returns `policy_decision=GO`

## Phase 6 Evidence
- Root shell size: `12` LOC
- Extraction modules: `23`, `54`, `31`, `62`, `32` LOC
- Support modules: `8`, `53`, `13`, `17`, `8`, `47`, `61`, `189` LOC
- Real target exercises the wired root shell and extracted sibling modules without behavior drift

## Command Evidence
- `cargo test -p kamn-core --test e2e_live_workflow_lane_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test e2e_live_workflow_lane -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6808-touched-size.json`

## Deviations
- None.
