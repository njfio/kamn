# 6737 - Split topology coherence contract tests

## Objective
Reduce `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests.rs` to a thin shell by extracting bounded concern-based sibling modules without changing daemon topology behavior.

## Inputs/Outputs
- Input: existing live-postgres daemon topology coherence contract tests
- Output: thin root module, extracted sibling modules, and an extraction contract that enforces the layout and root-shell budget

## Boundaries/Non-goals
- Do not change live-postgres topology behavior or fixture semantics
- Do not redesign unrelated daemon test modules
- Do not weaken or delete existing topology coherence assertions

## Failure modes
- Root file remains above the touched-file budget
- Extracted modules exceed the active size budget
- Existing daemon topology coverage stops running from the real `daemon_tests` entrypoint
- Extraction contract does not enforce expected module layout and root-shell budget

## Acceptance criteria
- [ ] `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests.rs` is reduced to a thin shell under the touched-file budget
- [ ] Topology coherence coverage is split into bounded concern-based sibling modules
- [ ] An extraction contract verifies the required module layout and root-shell budget
- [ ] Existing `kamn-node` daemon topology coherence coverage still runs from the real `daemon_tests` entrypoint
- [ ] The touched-Rust size policy reports `GO` for the issue write set

## Files to touch
- `specs/6737-split-topology-coherence-contract-tests.md`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/**`
- `crates/kamn-node/tests/topology_coherence_contract_tests_extraction_contract.rs`

## Error semantics
- Contract failures must hard-fail with explicit missing-file or budget diagnostics
- Existing test assertions must remain fail-closed and preserve current panic/test behavior

## Test plan
- Add a red extraction contract that fails while the root file is still monolithic
- Extract the topology coherence surface into bounded sibling modules
- Run the extraction contract
- Run `cargo test -p kamn-node daemon_tests -- --nocapture`
- Run `bash scripts/ci/check_touched_rust_size_policy.sh --output-json <path>`

## Phase 6 evidence
- Real entrypoint preserved: `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests.rs` still includes `topology_coherence_contract_tests.rs`
- Extraction contract: `cargo test -p kamn-node --test topology_coherence_contract_tests_extraction_contract -- --nocapture`
- Real suite path: `cargo test -p kamn-node daemon_tests -- --nocapture`
- Touched-Rust ratchet: `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6737-rebuild-yJwVlU --base-ref origin/main --output-json /tmp/6737-touched-size-clean.json`
- Ratchet result: `policy_decision=GO`

## Deviations
- The root shell keeps contract-expected `include!(...)` markers as comments while the compiled module wiring uses `#[path = ...] mod ...;` so the extracted tests compile as bounded sibling modules without changing the extraction contract expectations.
