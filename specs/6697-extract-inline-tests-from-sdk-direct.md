# Objective

Extract the inline `#[cfg(test)] mod tests` block from `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs` into sibling test modules so production code and test code are separated, the driver file size is reduced, and the extraction stays within the touched-Rust size ratchet.

# Inputs/Outputs

## Inputs
- Existing inline tests embedded in `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- Existing `kamn-e2e-harness` test entrypoints that exercise SDK-direct driver behavior
- Repo file-size and touched-Rust size policy constraints

## Outputs
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs` without inline test definitions
- A sibling `sdk_direct_tests.rs` module entry file
- Extracted leaf/support files under `crates/kamn-e2e-harness/src/drivers/sdk_direct_tests/`
- A contract test that rejects inline test reintroduction or layout drift

# Boundaries/Non-goals

- Do not change `SdkDirectDriver` runtime behavior
- Do not change public APIs or scenario semantics
- Do not refactor unrelated production helpers unless required to keep touched files within policy
- Do not introduce new dependencies

# Failure modes

- Inline `#[cfg(test)] mod tests` remains in `sdk_direct.rs`
- Extracted module root or support layout is missing
- Any new extracted file exceeds the 200 LOC limit for touched code
- `sdk_direct.rs` fails to shrink to the staged target after extraction
- Existing SDK-direct driver tests regress after the move

# Acceptance criteria

- [ ] `sdk_direct.rs` no longer contains an inline `#[cfg(test)] mod tests` block
- [ ] `sdk_direct.rs` declares a sibling path-based test module entrypoint
- [ ] Extracted tests live under `crates/kamn-e2e-harness/src/drivers/sdk_direct_tests/`
- [ ] All extracted leaf/support files are <= 200 LOC
- [ ] `sdk_direct.rs` is reduced to <= 1800 LOC after the initial extraction wave
- [ ] A contract test fails if inline tests return or the staged layout regresses
- [ ] Relevant `kamn-e2e-harness` tests pass after extraction
- [ ] Touched-Rust size policy passes on the issue branch

# Files to touch

- `specs/6697-extract-inline-tests-from-sdk-direct.md`
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/sdk_direct_tests.rs`
- `crates/kamn-e2e-harness/src/drivers/sdk_direct_tests/**`
- `crates/kamn-e2e-harness/tests/sdk_direct_inline_test_extraction_contract.rs`

# Error semantics

- Contract tests fail hard with exact missing-path or size-regression details
- No fallback to inline tests or alternate module layouts
- Existing driver tests keep their current assertion behavior; extraction changes only module placement and helpers required for bounded files

# Test plan

1. Add a red extraction contract that asserts the new module layout and staged size cap and fails while tests remain inline.
2. Extract the inline tests into bounded sibling files until the contract passes.
3. Run focused `kamn-e2e-harness` tests covering extracted SDK-direct driver behavior.
4. Run touched-Rust size policy on the full issue write set.
