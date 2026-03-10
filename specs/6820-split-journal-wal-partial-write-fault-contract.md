# 6820 Split journal WAL partial write fault contract tests

## Objective
Split `crates/kamn-core/tests/journal_wal_partial_write_fault_contract.rs` into a thin root shell plus bounded concern modules while preserving the existing WAL partial-write fault coverage.

## Inputs/Outputs
- Input: the current monolithic `journal_wal_partial_write_fault_contract.rs` test target on `main`
- Output: a root shell that wires bounded sibling modules for fixture parsing, store-case runners, matrix execution, and taxonomy/performance checks plus shared support

## Boundaries/Non-goals
- Do not change production WAL recovery behavior
- Do not add new dependencies
- Do not weaken or delete current assertions
- Do not alter public APIs or production error semantics

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared support drifts from the original fixture parser or fault runner semantics
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [ ] `crates/kamn-core/tests/journal_wal_partial_write_fault_contract.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] Root shell wires bounded sibling modules for fixture parsing, store-case runners, matrix execution, and taxonomy/performance checks plus shared support
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-core --test journal_wal_partial_write_fault_contract_extraction_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test journal_wal_partial_write_fault_contract -- --nocapture` passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/journal-wal-partial-write-fault-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6820-split-journal-wal-partial-write-fault-contract.md`
- `crates/kamn-core/tests/journal_wal_partial_write_fault_contract.rs`
- `crates/kamn-core/tests/journal_wal_partial_write_fault_contract_extraction_contract.rs`
- `crates/kamn-core/tests/journal_wal_partial_write_fault_contract/**`

## Error semantics
- Tests remain fail-closed and preserve the current panic/assert behavior for WAL partial-write drift
- Shared helpers may panic with explicit messages when fixture parsing or store setup fails
- No silent fallbacks or weakened fault assertions are introduced

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the target into bounded sibling modules plus shared support
3. Run the extraction contract target
4. Run the real `journal_wal_partial_write_fault_contract` target
5. Run the touched-Rust size checker against `origin/main`
