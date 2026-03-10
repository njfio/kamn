# Objective

Extract the S-06 through S-10 live probe implementations out of `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs` into bounded sibling production modules so the root CLI-scripted driver keeps shrinking while preserving real CLI runtime wiring and behavior.

# Inputs/Outputs

## Inputs
- Existing production live probe implementations for S-06 through S-10 in `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- Existing `kamn-e2e-harness` tests that exercise CLI-scripted driver behavior, validators, and runtime configuration
- The tranche-one extraction layout already merged for `cli_scripted`
- Existing touched-Rust size policy and file-size limits for touched files

## Outputs
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs` delegating S-06 through S-10 live probe work to sibling modules
- New bounded production modules under `crates/kamn-e2e-harness/src/drivers/cli_scripted/live_probe_tranche_two/`
- A contract test that prevents tranche-two layout regressions and staged root-size regressions
- Updated spec evidence for the extracted production tranche

# Boundaries/Non-goals

- Do not change CLI-scripted scenario semantics, env var contracts, or external command behavior
- Do not refactor S-11 through S-15 in this issue
- Do not introduce new dependencies

# Failure modes

- `cli_scripted.rs` keeps the S-06 through S-10 live probe implementations inline
- Extracted production tranche files are missing or exceed 200 LOC
- `cli_scripted.rs` does not shrink below the staged root cap after extraction
- CLI runtime wiring no longer delegates to the extracted tranche
- Existing CLI-scripted driver tests regress after extraction
- Touched-Rust size policy fails on the branch

# Acceptance criteria

- [ ] S-06 through S-10 live probe implementations are extracted from `cli_scripted.rs` into sibling production modules
- [ ] Any tranche support files touched by the extraction remain <= 200 LOC
- [ ] `cli_scripted.rs` preserves real production wiring and delegates to the extracted tranche
- [ ] `cli_scripted.rs` is reduced below a staged tranche-2 root cap of `1200` LOC enforced by contract tests
- [ ] A contract test fails if the tranche layout regresses or the staged root cap is exceeded
- [ ] Relevant `kamn-e2e-harness` CLI-scripted tests pass after extraction
- [ ] Touched-Rust size policy passes on the issue branch

# Files to touch

- `specs/6707-split-cli-scripted-live-probe-tranche-2.md`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted/live_probe_tranche_two/**`
- `crates/kamn-e2e-harness/tests/**` or `crates/kamn-e2e-harness/src/drivers/**` contract coverage needed for the tranche

# Error semantics

- Contract tests fail hard with exact missing-path, file-size, and staged-root-cap details
- Extraction keeps the current hard-fail error behavior for live probe implementations and validators
- No fallback to inline production probe definitions or alternate wiring layouts

# Test plan

1. Add a red contract that asserts the new S-06 through S-10 production tranche layout and the staged root cap.
2. Extract the second production tranche into bounded sibling modules until the contract passes.
3. Run focused `kamn-e2e-harness` tests covering CLI-scripted driver behavior and runtime contracts.
4. Run touched-Rust size policy on the full issue write set.

# Integration evidence

- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs` now delegates S-06 through S-10 into `src/drivers/cli_scripted/live_probe_tranche_two.rs`
- Root driver file size after extraction: `1132` LOC, below the staged tranche-2 contract cap of `1200`
- Extracted tranche-two files are all within the `<= 200` LOC touched-file limit
- Verified on branch head `433b74cbe912b48acc4e35b6c2d08623ebc086b5`:
  - `cargo test -p kamn-e2e-harness --test cli_scripted_tranche_two_extraction_contract -- --nocapture`
  - `cargo test -p kamn-e2e-harness cli_scripted -- --nocapture`
  - `cargo test -p kamn-e2e-harness --test command_contract -- --nocapture`
  - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6707-touched-size-post-refactor.json`
