# Objective

Extract the inline `#[cfg(test)] mod tests` block from `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs` into sibling test modules so production code and test code are separated, the driver file size is reduced, and the extraction stays within the touched-Rust size ratchet.

# Inputs/Outputs

## Inputs
- Existing inline tests embedded in `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- Existing `kamn-e2e-harness` test entrypoints that exercise CLI scripted driver behavior
- Repo file-size and touched-Rust size policy constraints

## Outputs
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs` without inline test definitions
- A sibling `cli_scripted_tests.rs` module entry file
- Extracted leaf/support files under `crates/kamn-e2e-harness/src/drivers/cli_scripted_tests/`
- A contract test that rejects inline test reintroduction or layout drift

# Boundaries/Non-goals

- Do not change `CliScriptedDriver` runtime behavior
- Do not change public APIs or command semantics
- Do not refactor unrelated production helpers unless required to keep touched files within policy
- Do not introduce new dependencies

# Failure modes

- Inline `#[cfg(test)] mod tests` remains in `cli_scripted.rs`
- Extracted module root or support layout is missing
- Any new extracted file exceeds the 200 LOC limit for touched code
- `cli_scripted.rs` fails to shrink to the staged target after extraction
- Existing CLI scripted driver tests regress after the move

# Acceptance criteria

- [ ] `cli_scripted.rs` no longer contains an inline `#[cfg(test)] mod tests` block
- [ ] `cli_scripted.rs` declares a sibling path-based test module entrypoint
- [ ] Extracted tests live under `crates/kamn-e2e-harness/src/drivers/cli_scripted_tests/`
- [ ] All extracted leaf/support files are <= 200 LOC
- [ ] `cli_scripted.rs` is reduced to <= 2600 LOC after the initial extraction wave
- [ ] A contract test fails if inline tests return or the staged layout regresses
- [ ] Relevant `kamn-e2e-harness` tests pass after extraction
- [ ] Touched-Rust size policy passes on the issue branch

# Files to touch

- `specs/6695-extract-inline-tests-from-cli-scripted.md`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted_tests.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted_tests/**`
- `crates/kamn-e2e-harness/tests/cli_scripted_inline_test_extraction_contract.rs`

# Error semantics

- Contract tests fail hard with exact missing-path or size-regression details
- No fallback to inline tests or alternate module layouts
- Existing driver tests keep their current assertion behavior; extraction changes only module placement and helpers required for bounded files

# Test plan

1. Add a red extraction contract that asserts the new module layout and staged size cap and fails while tests remain inline.
2. Extract the inline tests into bounded sibling files until the contract passes.
3. Run focused `kamn-e2e-harness` tests covering extracted CLI scripted driver behavior.
4. Run touched-Rust size policy on the full issue write set.

# Outcome

- `cli_scripted.rs` no longer contains the inline `#[cfg(test)] mod tests` block.
- `cli_scripted.rs` now wires the extracted test surface through `#[path = "cli_scripted_tests.rs"] mod cli_scripted_tests;`.
- `cli_scripted.rs` is now 2,116 LOC, down from 3,478 LOC.
- All extracted leaf/support files are within the 200 LOC touched-code limit.

# Evidence

- `cargo test -p kamn-e2e-harness cli_scripted -- --nocapture`
- `cargo test -p kamn-e2e-harness --test cli_scripted_inline_test_extraction_contract -- --nocapture`
- `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6695-touched-size-refactor.json`

# Deviations

- No behavior deviations from the issue scope.
- The staged root-file budget remains above the repo-wide 200 LOC target because this issue only extracts the inline test surface; the residual production file is 2,116 LOC and should be reduced further in follow-up issues.
