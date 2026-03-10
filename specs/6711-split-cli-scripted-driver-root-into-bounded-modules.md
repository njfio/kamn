# Objective

Reduce `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs` to a thin public wiring surface by extracting the residual driver-core construction/execution logic and CLI command helper surface into bounded sibling modules without changing CLI-scripted runtime behavior.

# Inputs/Outputs

## Inputs
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs` at 387 LOC after the live-probe tranche extractions
- Existing `kamn-e2e-harness` CLI-scripted tests and command contracts
- Existing sibling tranche modules under `crates/kamn-e2e-harness/src/drivers/cli_scripted/`
- Existing touched-Rust size policy and file/function size limits

## Outputs
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs` reduced to <= 200 LOC
- New bounded sibling modules for the residual root responsibilities, expected seams:
  - `driver_core.rs` for driver construction and execution
  - `command_support.rs` for CLI command spawning and output parsing
- Contract coverage that fails if the root grows back above the staged cap or the extracted layout regresses
- Updated spec evidence for the extracted root surface

# Boundaries/Non-goals

- Do not change CLI-scripted scenario semantics, env var contracts, or external command behavior
- Do not refactor other drivers in this issue
- Do not introduce new dependencies

# Failure modes

- `cli_scripted.rs` remains above the 200 LOC cap
- Driver construction or execution logic stays inline in the root file
- CLI command helper paths stop failing hard on spawn, empty output, or unexpected success/failure cases
- Existing CLI-scripted driver tests regress after extraction
- Touched-Rust size policy fails on the issue branch

# Acceptance criteria

- [ ] `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs` is reduced to <= 200 LOC
- [ ] residual driver-core responsibilities are extracted into bounded sibling modules
- [ ] real CLI-scripted runtime wiring remains unchanged
- [ ] a contract test fails if the root grows above the staged cap or the extracted module layout regresses
- [ ] relevant `kamn-e2e-harness` CLI-scripted tests and command contracts pass after extraction
- [ ] touched-Rust size policy passes on the issue branch

# Files to touch

- `specs/6711-split-cli-scripted-driver-root-into-bounded-modules.md`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted/**`
- `crates/kamn-e2e-harness/tests/**` or `crates/kamn-e2e-harness/src/drivers/**` contract coverage needed for the extraction

# Error semantics

- Extraction preserves the current hard-fail command-spawn, exit-status, and empty-output behavior
- Contract tests fail hard with exact missing-path, file-size, and staged-root-cap details
- No fallback to inline helper implementations or alternate driver wiring layouts

# Test plan

1. Add a red contract that asserts the new residual-root layout and a <= 200 LOC root cap.
2. Extract the driver-core and/or command-support seams until the contract passes.
3. Run focused `kamn-e2e-harness` CLI-scripted tests and command contracts.
4. Run touched-Rust size policy on the issue write set.
