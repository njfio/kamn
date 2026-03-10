# 6822 Split main module extraction contract tests

## Objective
Split `crates/kamn-node/tests/main_module_extraction_contract.rs` into a thin root shell plus bounded concern modules while preserving the existing main-module extraction contract coverage.

## Inputs/Outputs
- Input: the current monolithic `main_module_extraction_contract.rs` test target on `main`
- Output: a root shell that wires bounded sibling modules for main-root declarations, extracted runtime/model/report boundaries, runtime orchestration boundaries, and test-shell budget guards plus shared support

## Boundaries/Non-goals
- Do not change production `kamn-node` runtime behavior
- Do not add new dependencies
- Do not weaken or delete current assertions
- Do not alter public APIs or runtime error semantics

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared support drifts from the original repo-file reading behavior
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [x] `crates/kamn-node/tests/main_module_extraction_contract.rs` is reduced to a thin root shell at or below 180 LOC
- [x] Root shell wires bounded sibling modules for the current main-module extraction concerns and shared support
- [x] All extracted files touched by the split remain at or below 200 LOC
- [x] `cargo test -p kamn-node --test main_module_extraction_contract_extraction_contract -- --nocapture` passes
- [x] `cargo test -p kamn-node --test main_module_extraction_contract -- --nocapture` passes
- [x] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/main-module-extraction-contract-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6822-split-main-module-extraction-contract.md`
- `crates/kamn-node/tests/main_module_extraction_contract.rs`
- `crates/kamn-node/tests/main_module_extraction_contract_extraction_contract.rs`
- `crates/kamn-node/tests/main_module_extraction_contract/**`

## Error semantics
- Tests remain fail-closed and preserve the current panic/assert behavior for extraction drift
- Shared helpers may panic with explicit messages when repo files cannot be read
- No silent fallbacks or weakened contract assertions are introduced

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the target into bounded sibling modules plus shared support
3. Run the extraction contract target
4. Run the real `main_module_extraction_contract` target
5. Run the touched-Rust size checker against `origin/main`

## Phase 6 evidence
- `cargo test -p kamn-node --test main_module_extraction_contract_extraction_contract -- --nocapture`
- `cargo test -p kamn-node --test main_module_extraction_contract -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/main-module-extraction-contract-size.json`
- Result: `policy_decision=GO`

## Deviations
- None
