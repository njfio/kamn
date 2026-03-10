# 6742 - Split topology mapping contract tests

## Objective
Reduce `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_mapping_contract_tests.rs` to a thin shell by extracting bounded concern-based sibling modules without changing daemon topology behavior.

## Inputs/Outputs
- Input: existing live-postgres daemon topology mapping contract tests
- Output: thin root module, extracted sibling modules, and an extraction contract that enforces the layout and root-shell budget

## Boundaries/Non-goals
- Do not change live-postgres topology behavior or fixture semantics
- Do not redesign unrelated daemon test modules
- Do not weaken or delete existing topology mapping assertions

## Failure modes
- Root file remains above the touched-file budget
- Extracted modules exceed the active size budget
- Existing daemon topology mapping coverage stops running from the real `daemon_tests` entrypoint
- Extraction contract does not enforce expected module layout and root-shell budget

## Acceptance criteria
- [ ] `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_mapping_contract_tests.rs` is reduced to a thin shell under the touched-file budget
- [ ] Topology mapping coverage is split into bounded concern-based sibling modules
- [ ] An extraction contract verifies the required module layout and root-shell budget
- [ ] Existing `kamn-node` daemon topology mapping coverage still runs from the real `daemon_tests` entrypoint
- [ ] The touched-Rust size policy reports `GO` for the issue write set

## Files to touch
- `specs/6742-split-topology-mapping-contract-tests.md`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_mapping_contract_tests.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_mapping_contract_tests/**`
- `crates/kamn-node/tests/topology_mapping_contract_tests_extraction_contract.rs`

## Error semantics
- Contract failures must hard-fail with explicit missing-file or budget diagnostics
- Existing test assertions must remain fail-closed and preserve current panic/test behavior

## Test plan
- Add a red extraction contract that fails while the root file is still monolithic
- Extract the topology mapping surface into bounded sibling modules
- Run the extraction contract
- Run `cargo test -p kamn-node daemon_tests -- --nocapture`
- Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root <repo> --base-ref origin/main --output-json <path>`

## Planned extraction seams
- host-pair identity
- host-pair directionality
- host-pair mapping rows
- lane-set mapping rows
- lane-count mapping rows
- host-mode mapping rows
- host-cardinality mapping rows
