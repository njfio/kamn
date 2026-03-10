# Objective

Extract the S-11 through S-15 live probe implementations out of `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs` into bounded sibling production modules so the root CLI-scripted driver shrinks to the residual non-live surface while preserving real CLI runtime wiring and behavior.

# Inputs/Outputs

## Inputs
- Existing production live probe implementations for S-11 through S-15 in `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- Existing `kamn-e2e-harness` tests that exercise CLI-scripted driver behavior, validators, and runtime configuration
- The tranche-one and tranche-two extraction layouts already merged for `cli_scripted`
- Existing touched-Rust size policy and file-size limits for touched files

## Outputs
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs` delegating S-11 through S-15 live probe work to sibling modules
- New bounded production modules under `crates/kamn-e2e-harness/src/drivers/cli_scripted/live_probe_tranche_three/`
- A contract test that prevents tranche-three layout regressions and staged root-size regressions
- Updated spec evidence for the extracted final production tranche

# Boundaries/Non-goals

- Do not change CLI-scripted scenario semantics, env var contracts, or external command behavior
- Do not refactor unrelated drivers in this issue
- Do not introduce new dependencies

# Failure modes

- `cli_scripted.rs` keeps the S-11 through S-15 live probe implementations inline
- Extracted production tranche files are missing or exceed 200 LOC
- `cli_scripted.rs` does not shrink below the staged root cap after extraction
- CLI runtime wiring no longer delegates to the extracted tranche
- Existing CLI-scripted driver tests regress after extraction
- Touched-Rust size policy fails on the branch

# Acceptance criteria

- [ ] S-11 through S-15 live probe implementations are extracted from `cli_scripted.rs` into sibling production modules
- [ ] Any tranche support files touched by the extraction remain <= 200 LOC
- [ ] `cli_scripted.rs` preserves real production wiring and delegates to the extracted tranche
- [ ] `cli_scripted.rs` is reduced below a staged tranche-3 root cap of `400` LOC enforced by contract tests
- [ ] A contract test fails if the tranche layout regresses or the staged root cap is exceeded
- [ ] Relevant `kamn-e2e-harness` CLI-scripted tests pass after extraction
- [ ] Touched-Rust size policy passes on the issue branch

# Files to touch

- `specs/6709-split-cli-scripted-live-probe-tranche-3.md`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted/live_probe_tranche_three/**`
- `crates/kamn-e2e-harness/tests/**` or `crates/kamn-e2e-harness/src/drivers/**` contract coverage needed for the tranche

# Error semantics

- Contract tests fail hard with exact missing-path, file-size, and staged-root-cap details
- Extraction keeps the current hard-fail error behavior for live probe implementations and validators
- No fallback to inline production probe definitions or alternate wiring layouts

# Test plan

1. Add a red contract that asserts the new S-11 through S-15 production tranche layout and the staged root cap.
2. Extract the final production tranche into bounded sibling modules until the contract passes.
3. Run focused `kamn-e2e-harness` tests covering CLI-scripted driver behavior and runtime contracts.
4. Run touched-Rust size policy on the full issue write set.
