# 6966-split-data-layer-m3-blind-index-search

## Objective
Reduce `crates/kamn-core/src/data_layer_m3_blind_index_search.rs` to a thin root shell by extracting deterministic M3 blind-index concerns into bounded modules without changing blind-index search, metadata query, retrieval projection, or determinism semantics.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/data_layer_m3_blind_index_search.rs`
  - existing M3 search callers and tests on current `main`
- Outputs:
  - thin root `data_layer_m3_blind_index_search.rs` shell
  - bounded modules for M3 models, catalog operations, hashing/normalization, validation/support, errors, and tests
  - hard-fail extraction contract guarding the split
  - updated spec evidence for the decomposition

## Boundaries/Non-goals
- Do not change blind-index query semantics, metadata filtering semantics, retrieval projection semantics, or determinism report behavior.
- Do not add dependencies.
- Do not do broad warning cleanup outside the touched M3 surface.
- Do not rewrite content-retrieval contracts beyond what is needed to keep the extracted module graph compiling.

## Failure modes
- Root `data_layer_m3_blind_index_search.rs` remains over the active file budget.
- Extracted modules exceed the file budget.
- Exact-match blind-index search stops enforcing owner scope or stable ordering.
- Metadata queries stop enforcing validation or timestamp bound checks.
- Retrieval projection stops failing hard on missing CID mappings or invalid retrieval requests.
- Determinism evaluation stops reporting drift/stability correctly.
- Inline tests are lost or no longer compile.

## Acceptance criteria
- [x] `crates/kamn-core/src/data_layer_m3_blind_index_search.rs` is reduced to a thin root shell within the active file budget.
- [x] Concern-based modules are extracted under `crates/kamn-core/src/data_layer_m3_blind_index_search/`.
- [x] A hard-fail extraction contract fails closed if root markers, extracted files, or file budgets regress.
- [x] Existing M3 blind-index tests remain green on current `main`.
- [x] Touched-Rust size policy returns `policy_decision=GO`.

## Files to touch
- `crates/kamn-core/src/data_layer_m3_blind_index_search.rs`
- `crates/kamn-core/src/data_layer_m3_blind_index_search/*.rs`
- `crates/kamn-core/tests/data_layer_m3_blind_index_search_module_extraction_contract.rs`
- `specs/6966-split-data-layer-m3-blind-index-search.md`

## Error semantics
- Preserve current typed `DataLayerM3SearchError` behavior and `Result<_, DataLayerM3SearchError>` signatures.
- Keep hard-fail validation for malformed DIDs, field names, tokens, timestamp bounds, limit values, and missing CID bridge mappings.
- Do not add silent fallbacks or soften any existing validation path.

## Test plan
- Red extraction contract asserting root budget, root markers, extracted module presence, and extracted file budgets.
- Existing M3 unit tests from `data_layer_m3_blind_index_search.rs`.
- `cargo test -p kamn-core --test data_layer_m3_blind_index_search_module_extraction_contract -- --nocapture`.
- `cargo test -p kamn-core data_layer_m3_blind_index_search::tests:: --lib -- --nocapture`.
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6966-touched-size.json`.

## Final evidence
- Root shell size: `29` LOC in `crates/kamn-core/src/data_layer_m3_blind_index_search.rs`.
- Extracted module tree:
  - `models.rs`
  - `catalog.rs`
  - `catalog/{register,search,projection,determinism,metadata}.rs`
  - `hashing.rs`
  - `validation.rs`
  - `errors.rs`
  - `tests.rs`
- Verified commands:
  - `cargo test -p kamn-core --test data_layer_m3_blind_index_search_module_extraction_contract -- --nocapture`
  - `cargo test -p kamn-core data_layer_m3_blind_index_search::tests:: --lib -- --nocapture`
  - `cargo test -p kamn-core --test data_layer_m3_blind_index_search -- --nocapture`
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6966-touched-size.json`
- Touched-Rust result: `policy_decision=GO`.

## Deviations
- None in runtime behavior.
- The Phase 6 `integrate(6966)` commit is intentionally empty because this issue is an internal extraction. The verified integration path is the preserved `kamn-core` root shell plus the existing `data_layer_m3_blind_index_search` contract test target on current `main`.
