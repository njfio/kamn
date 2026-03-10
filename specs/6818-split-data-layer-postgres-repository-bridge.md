# 6818 Split data layer postgres repository bridge tests

## Objective
Split `crates/kamn-core/tests/data_layer_postgres_repository_bridge.rs` into a thin root shell plus bounded concern modules while preserving the existing PostgreSQL repository bridge coverage.

## Inputs/Outputs
- Input: the current monolithic `data_layer_postgres_repository_bridge.rs` test target on `main`
- Output: a root shell that wires bounded sibling modules for M0, M5, M6, and M7 bridge coverage plus shared fixtures/support

## Boundaries/Non-goals
- Do not change production PostgreSQL repository bridge behavior
- Do not add new dependencies
- Do not weaken or delete current assertions
- Do not alter public APIs or production error semantics

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared fixtures drift from the original deterministic bridge scenarios
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [x] `crates/kamn-core/tests/data_layer_postgres_repository_bridge.rs` is reduced to a thin root shell at or below 180 LOC
- [x] Root shell wires bounded sibling modules for M0, M5, M6, and M7 bridge concerns plus shared support
- [x] All extracted files touched by the split remain at or below 200 LOC
- [x] `cargo test -p kamn-core --test data_layer_postgres_repository_bridge_extraction_contract -- --nocapture` passes
- [x] `cargo test -p kamn-core --test data_layer_postgres_repository_bridge -- --nocapture` passes
- [x] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/data-layer-postgres-repository-bridge-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6818-split-data-layer-postgres-repository-bridge.md`
- `crates/kamn-core/tests/data_layer_postgres_repository_bridge.rs`
- `crates/kamn-core/tests/data_layer_postgres_repository_bridge_extraction_contract.rs`
- `crates/kamn-core/tests/data_layer_postgres_repository_bridge/**`

## Error semantics
- Tests remain fail-closed and preserve the current panic/assert behavior for repository bridge drift
- Shared helpers may panic with explicit messages when deterministic fixture setup fails
- No silent fallbacks or weakened policy assertions are introduced

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the target into bounded sibling modules plus shared support
3. Run the extraction contract target
4. Run the real `data_layer_postgres_repository_bridge` target
5. Run the touched-Rust size checker against `origin/main`

## Phase 6 evidence
- Root shell reduced to `10` LOC and now only wires extracted sibling modules plus shared support
- `cargo test -p kamn-core --test data_layer_postgres_repository_bridge_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test data_layer_postgres_repository_bridge -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/data-layer-postgres-repository-bridge-size.json`
- Final touched-Rust result: `policy_decision=GO`

## Deviations
- None
