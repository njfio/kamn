# 6733 Split live postgres matrix contract tests

## Objective
Split `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_matrix_contract_tests.rs` into bounded concern-based modules while preserving the existing live-postgres matrix coverage, deterministic ordering guarantees, and daemon Phase 6 runtime reason taxonomy assertions.

## Inputs/Outputs
- Input: the current live-postgres matrix contract test surface in `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_matrix_contract_tests.rs`
- Output: a thin root test shell plus bounded sibling modules for each logical matrix concern, with extraction contracts that ratchet the root shell and module layout

## Boundaries/Non-goals
- Do not change daemon runtime behavior or CLI argument semantics
- Do not weaken or delete live-postgres integration coverage
- Do not redesign the fixture helpers extracted in `#6731`
- Do not touch unrelated daemon or runtime test files outside the extraction surface unless required for imports/module wiring

## Failure modes
- Root file remains oversized after extraction
- Extracted leaf files exceed the 200 LOC policy
- Logical coverage drifts because a matrix/test cluster is omitted from the new module tree
- Canonical ordering or taxonomy assertions change during the move
- Module wiring compiles but no longer runs the extracted tests from the real crate entrypoint

## Acceptance criteria
- [ ] `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_matrix_contract_tests.rs` is reduced to a thin module shell under the touched-file budget
- [ ] The matrix contract surface is split into bounded modules grouped by logical concern rather than arbitrary line ranges
- [ ] An extraction contract verifies the required module layout and root-shell budget
- [ ] Existing live-postgres matrix tests still run from the real `kamn-node` test entrypoint
- [ ] Canonical ordering, CSV/taxonomy bridge, and permutation invariance assertions remain intact
- [ ] The touched-Rust size policy reports `GO` for the issue write set

## Files to touch
- `specs/6733-split-live-postgres-matrix-contract-tests.md`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_matrix_contract_tests.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_matrix_contract_tests/`
- `crates/kamn-node/tests/live_postgres_matrix_contract_tests_extraction_contract.rs`

## Error semantics
- Extraction contracts fail loudly with explicit missing-path, missing-marker, or size-budget assertions
- Existing test assertions keep current hard-fail behavior and must not introduce silent fallbacks
- Any required helper/import rewiring must preserve the current panic/assert behavior of the moved tests

## Test plan
1. Add a red extraction contract that fails while the root file still contains the inline matrix clusters and the module tree is absent
2. Extract the matrix clusters into bounded sibling modules and keep the root as a thin shell
3. Run `cargo test -p kamn-node --test live_postgres_matrix_contract_tests_extraction_contract -- --nocapture`
4. Run `cargo test -p kamn-node daemon_tests -- --nocapture`
5. Run `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /home/n/Code/kamn/tmp/6733-touched-size.json`
