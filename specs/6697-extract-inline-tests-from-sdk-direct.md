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

- [x] `sdk_direct.rs` no longer contains an inline `#[cfg(test)] mod tests` block
- [x] `sdk_direct.rs` declares a sibling path-based test module entrypoint
- [x] Extracted tests live under `crates/kamn-e2e-harness/src/drivers/sdk_direct_tests/`
- [x] All extracted leaf/support files are <= 200 LOC
- [x] `sdk_direct.rs` is reduced to <= 1800 LOC after the initial extraction wave
- [x] A contract test fails if inline tests return or the staged layout regresses
- [x] Relevant `kamn-e2e-harness` tests pass after extraction
- [x] Touched-Rust size policy passes on the issue branch

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

# Deviations

- Renamed the support helper leaf from `script_fixture_support.rs` to `probe_fixture_support.rs` during Phase 5 because the file stores endpoint/probe fixtures rather than script fixtures.

# Final evidence

- `sdk_direct.rs` reduced from `2548` LOC to `1645` LOC.
- Extracted files live under `crates/kamn-e2e-harness/src/drivers/sdk_direct_tests/`, including `base_contract_tests.rs`, `driver_path_contract_tests.rs`, `invalid_endpoint_probe_contract_tests.rs`, `live_probe_contract_tests.rs`, `payload_and_budget_contract_tests.rs`, `validator_contract_tests.rs`, and bounded support helpers.
- Clean detached verification worktree: `/tmp/kamn-6697-verify-iMVPpc`
- Verified commands:
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-e2e-harness sdk_direct -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-e2e-harness --test sdk_direct_inline_test_extraction_contract -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-e2e-harness --test sdk_direct_live_toggle_contract -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-e2e-harness --test command_contract -- --nocapture`
  - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6697-clean-touched-size.json`
