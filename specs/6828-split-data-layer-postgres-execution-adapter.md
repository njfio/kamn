# 6828 Split data layer postgres execution adapter tests

## Objective
Split `crates/kamn-core/tests/data_layer_postgres_execution_adapter.rs` into a thin root shell plus bounded concern modules while preserving the existing postgres execution adapter contract coverage.

## Inputs/Outputs
- Input: the current monolithic `data_layer_postgres_execution_adapter.rs` test target on `main`
- Output: a root shell that wires bounded sibling modules for shared runtime fixtures, migration/config guards, live insert/search paths, merkle batch lifecycle contracts, and orchestrator persistence flows

## Boundaries/Non-goals
- Do not change production data-layer execution adapter behavior
- Do not add new dependencies
- Do not weaken or delete current assertions
- Do not alter public APIs or runtime error semantics

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared support drifts from the current live-postgres fixture behavior
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [x] `crates/kamn-core/tests/data_layer_postgres_execution_adapter.rs` is reduced to a thin root shell at or below 180 LOC
- [x] Root shell wires bounded sibling modules for the current execution-adapter contract concerns and shared support
- [x] All extracted files touched by the split remain at or below 200 LOC
- [x] `cargo test -p kamn-core --test data_layer_postgres_execution_adapter -- --nocapture` passes
- [x] `cargo test -p kamn-core --test data_layer_postgres_execution_adapter_extraction_contract -- --nocapture` passes
- [x] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/data-layer-postgres-execution-adapter-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6828-split-data-layer-postgres-execution-adapter.md`
- `crates/kamn-core/tests/data_layer_postgres_execution_adapter.rs`
- `crates/kamn-core/tests/data_layer_postgres_execution_adapter_extraction_contract.rs`
- `crates/kamn-core/tests/data_layer_postgres_execution_adapter/**`

## Error semantics
- Tests remain fail-closed and preserve the current panic/assert behavior for execution-adapter drift
- Shared helpers may panic with explicit messages when live postgres fixtures or runtime construction fail
- No silent fallbacks or weakened contract assertions are introduced

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the target into bounded sibling modules plus shared support where needed
3. Run the extraction contract target
4. Run the real `data_layer_postgres_execution_adapter` target
5. Run the touched-Rust size checker against `origin/main`

## Phase 6 evidence
- `cargo test -p kamn-core --test data_layer_postgres_execution_adapter_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test data_layer_postgres_execution_adapter -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/data-layer-postgres-execution-adapter-size.json`
- Result: `policy_decision=GO`
- Integration note: the extracted layout is exercised through the real `data_layer_postgres_execution_adapter` test target with all 9 live/guard/orchestrator checks still wired and passing

## Deviations
- None
