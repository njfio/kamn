# 6899-split-runtime-tests

## Objective
Split `crates/kamn-core/src/runtime_tests.rs` into bounded concern-based modules while preserving current runtime wiring regression coverage, lifecycle/backpressure behavior tests, authenticated peer-frame coverage, planner/recovery/construct-lock coverage, and quorum/watchdog coverage.

## Inputs/Outputs
- Input: current `crates/kamn-core/src/runtime_tests.rs` monolithic test source plus existing dedicated sibling modules `runtime_tests_snapshot_store.rs` and `runtime_tests_network_fault.rs`
- Output: a thin root shell plus bounded sibling test modules for the remaining runtime test domains
- Output: a hard-fail extraction contract enforcing the new layout

## Boundaries/Non-goals
- Do not change production runtime behavior or public API
- Do not redesign existing `runtime_tests_snapshot_store.rs` or `runtime_tests_network_fault.rs` beyond wiring them through the split root
- Do not add new runtime features or new dependencies

## Failure modes
- Extraction contract passes while `runtime_tests.rs` remains oversized or expected modules are missing
- Runtime wiring regression markers drift during extraction
- Lifecycle/backpressure, peer-frame, planner/recovery/lock, or quorum/watchdog behavior changes silently
- Any touched extracted file exceeds the touched-Rust size policy
- Final branch still fails touched-Rust size policy

## Acceptance criteria
- [ ] `crates/kamn-core/src/runtime_tests.rs` becomes a thin root shell under the active file-size policy
- [ ] sibling modules are introduced for root wiring/regression markers, lifecycle/backpressure, authenticated peer-frame, planner/recovery/construct-lock, and quorum/watchdog coverage
- [ ] existing dedicated snapshot-store and network-fault runtime test modules remain wired through the root shell
- [ ] no touched extracted file exceeds the active touched-Rust size policy
- [ ] a hard-fail extraction contract exists and passes
- [ ] existing runtime test coverage still passes after the split
- [ ] touched-Rust size policy returns `GO` on the final branch

## Files to touch
- `crates/kamn-core/src/runtime_tests.rs`
- `crates/kamn-core/src/runtime_tests/`
- `crates/kamn-core/tests/runtime_tests_module_extraction_contract.rs`
- `specs/6899-split-runtime-tests.md`

## Error semantics
- No new fallback paths or swallowed failures in test helpers
- Regression marker failures remain fail-closed assertions with explicit messages
- Existing runtime error type expectations remain unchanged

## Test plan
- Add a red extraction contract that fails while the root file remains oversized and the expected module layout is missing
- Run the extraction contract target
- Run runtime test coverage after the split
- Run touched-Rust size policy on the final branch
