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
- [x] `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_matrix_contract_tests.rs` is reduced to a thin module shell under the touched-file budget
- [x] The matrix contract surface is split into bounded modules grouped by logical concern rather than arbitrary line ranges
- [x] An extraction contract verifies the required module layout and root-shell budget
- [x] Existing live-postgres matrix tests still run from the real `kamn-node` test entrypoint
- [x] Canonical ordering, CSV/taxonomy bridge, and permutation invariance assertions remain intact
- [x] The touched-Rust size policy reports `GO` for the issue write set

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

## Implementation summary
- Reduced `live_postgres_matrix_contract_tests.rs` from `1030` LOC to `11` LOC
- Split the matrix surface into bounded files for:
  - env/gate execution
  - projection/taxonomy bridge
  - load profiles
  - role profiles
  - role pairs
  - symmetric parallel role-pair lanes
  - asymmetric parallel lanes
  - lane order/permutation invariance
- Added `support.rs` to centralize live-postgres setup and repeated projection assertions so the touched-file function budget stays green
- Added `live_postgres_matrix_contract_tests_extraction_contract.rs` to ratchet the root shell and module layout

## Evidence
- `cargo test -p kamn-node --test live_postgres_matrix_contract_tests_extraction_contract -- --nocapture`
- `cargo test -p kamn-node daemon_tests -- --nocapture`
- `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /home/n/Code/kamn/tmp/6733-touched-size-with-support.json`
- Measured extracted file sizes:
  - root shell: `11` LOC
  - `env_gate_execution_contract_tests.rs`: `69` LOC
  - `projection_taxonomy_contract_tests.rs`: `70` LOC
  - `load_profile_contract_tests.rs`: `48` LOC
  - `role_profile_contract_tests.rs`: `48` LOC
  - `role_pair_contract_tests.rs`: `31` LOC
  - `parallel_role_pair_lane_contract_tests.rs`: `34` LOC
  - `asymmetric_parallel_lane_contract_tests.rs`: `34` LOC
  - `parallel_lane_invariance_contract_tests/order_invariance_contract_tests.rs`: `45` LOC
  - `parallel_lane_invariance_contract_tests/permutation_invariance_contract_tests.rs`: `77` LOC
  - `support.rs`: `195` LOC

## Deviations
- The first green extraction used line-slice boundaries that left trailing `#[test]` attributes in several child files; this was corrected before the final green/refactor verification and no coverage was removed.
