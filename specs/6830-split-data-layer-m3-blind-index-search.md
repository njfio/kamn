# 6830 Split data layer m3 blind index search tests

## Objective
Split `crates/kamn-core/tests/data_layer_m3_blind_index_search.rs` into a thin root shell plus bounded concern modules while preserving the existing blind-index search contract coverage.

## Inputs/Outputs
- Input: the current monolithic `data_layer_m3_blind_index_search.rs` test target on `main`
- Output: a root shell that wires bounded sibling modules for shared record/catalog fixtures, blind-index search contracts, metadata search contracts, and determinism/retrieval projection contracts

## Boundaries/Non-goals
- Do not change production blind-index search behavior
- Do not add new dependencies
- Do not weaken or delete current assertions
- Do not alter public APIs or runtime error semantics

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared support drifts from the current record/catalog fixture behavior
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [ ] `crates/kamn-core/tests/data_layer_m3_blind_index_search.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] Root shell wires bounded sibling modules for the current blind-index search contract concerns and shared support
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-core --test data_layer_m3_blind_index_search -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test data_layer_m3_blind_index_search_extraction_contract -- --nocapture` passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/data-layer-m3-blind-index-search-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6830-split-data-layer-m3-blind-index-search.md`
- `crates/kamn-core/tests/data_layer_m3_blind_index_search.rs`
- `crates/kamn-core/tests/data_layer_m3_blind_index_search_extraction_contract.rs`
- `crates/kamn-core/tests/data_layer_m3_blind_index_search/**`

## Error semantics
- Tests remain fail-closed and preserve the current panic/assert behavior for blind-index/search drift
- Shared helpers may panic with explicit messages when catalog fixtures or projection expectations are invalid
- No silent fallbacks or weakened contract assertions are introduced

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the target into bounded sibling modules plus shared support where needed
3. Run the extraction contract target
4. Run the real `data_layer_m3_blind_index_search` target
5. Run the touched-Rust size checker against `origin/main`
