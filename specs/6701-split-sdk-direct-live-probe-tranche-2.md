# Objective

Extract the S-06 through S-10 live probe implementations out of `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs` into bounded sibling production modules so the root driver continues shrinking while preserving real SDK-direct runtime wiring and behavior.

# Inputs/Outputs

## Inputs
- Existing production live probe implementations for S-06 through S-10 in `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- Existing `kamn-e2e-harness` tests that exercise SDK-direct driver behavior and runtime configuration
- Existing touched-Rust size policy and file-size limits for touched files

## Outputs
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs` delegating S-06 through S-10 live probe work to sibling modules
- New bounded production modules under `crates/kamn-e2e-harness/src/drivers/sdk_direct/`
- A contract test that prevents tranche-layout regressions and staged root-size regressions
- Updated spec evidence for the extracted production tranche

# Boundaries/Non-goals

- Do not change SDK-direct scenario semantics, env var contracts, or public APIs
- Do not extract S-11 through S-15 in this issue
- Do not refactor unrelated driver orchestration outside the selected tranche unless required by the touched-size ratchet
- Do not introduce new dependencies

# Failure modes

- `sdk_direct.rs` keeps the S-06 through S-10 live probe implementations inline
- Extracted production tranche files are missing or exceed 200 LOC
- `sdk_direct.rs` does not shrink below the staged root cap after extraction
- SDK-direct runtime wiring no longer delegates to the extracted tranche
- Existing SDK-direct driver tests regress after extraction
- Touched-Rust size policy fails on the branch

# Acceptance criteria

- [ ] S-06 through S-10 live probe implementations are extracted from `sdk_direct.rs` into sibling production modules
- [ ] Any tranche support files touched by the extraction remain <= 200 LOC
- [ ] `sdk_direct.rs` preserves real production wiring and delegates to the extracted tranche
- [ ] `sdk_direct.rs` is reduced to <= 1000 LOC after the tranche extraction
- [ ] A contract test fails if the tranche layout regresses or the staged root cap is exceeded
- [ ] Relevant `kamn-e2e-harness` SDK-direct tests pass after extraction
- [ ] Touched-Rust size policy passes on the issue branch

# Files to touch

- `specs/6701-split-sdk-direct-live-probe-tranche-2.md`
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/sdk_direct/**`
- `crates/kamn-e2e-harness/tests/**` or `crates/kamn-e2e-harness/src/drivers/**` contract coverage needed for the tranche

# Error semantics

- Contract tests fail hard with exact missing-path, file-size, and staged-root-cap details
- Extraction keeps the current hard-fail error behavior for live probe implementations and validators
- No fallback to inline production probe definitions or alternate wiring layouts

# Test plan

1. Add a red contract that asserts the new S-06 through S-10 production tranche layout and the staged root cap.
2. Extract the second production tranche into bounded sibling modules until the contract passes.
3. Run focused `kamn-e2e-harness` tests covering SDK-direct driver behavior and command/runtime contracts.
4. Run touched-Rust size policy on the full issue write set.
