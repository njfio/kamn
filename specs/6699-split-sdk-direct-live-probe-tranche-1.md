# Objective

Extract the S-01 through S-05 live probe implementations out of `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs` into bounded sibling production modules so the root driver shrinks further while preserving real SDK-direct runtime wiring and behavior.

# Inputs/Outputs

## Inputs
- Existing production live probe implementations for S-01 through S-05 in `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- Existing `kamn-e2e-harness` tests that exercise SDK-direct driver behavior and runtime configuration
- Existing touched-Rust size policy and file-size limits for touched files

## Outputs
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs` delegating S-01 through S-05 live probe work to sibling modules
- New bounded production modules under `crates/kamn-e2e-harness/src/drivers/sdk_direct/`
- A contract test that prevents tranche-layout regressions and staged root-size regressions
- Updated spec evidence for the extracted production tranche

# Boundaries/Non-goals

- Do not change SDK-direct scenario semantics, env var contracts, or public APIs
- Do not extract S-06 through S-15 in this issue
- Do not refactor unrelated driver orchestration outside the selected tranche unless required by the touched-size ratchet
- Do not introduce new dependencies

# Failure modes

- `sdk_direct.rs` keeps the S-01 through S-05 live probe implementations inline
- Extracted production tranche files are missing or exceed 200 LOC
- `sdk_direct.rs` does not shrink below the staged root cap after extraction
- SDK-direct runtime wiring no longer delegates to the extracted tranche
- Existing SDK-direct driver tests regress after extraction
- Touched-Rust size policy fails on the branch

# Acceptance criteria

- [x] S-01 through S-05 live probe implementations are extracted from `sdk_direct.rs` into sibling production modules
- [x] Any tranche support files touched by the extraction remain <= 200 LOC
- [x] `sdk_direct.rs` preserves real production wiring and delegates to the extracted tranche
- [x] `sdk_direct.rs` is reduced to <= 1300 LOC after the tranche extraction
- [x] A contract test fails if the tranche layout regresses or the staged root cap is exceeded
- [x] Relevant `kamn-e2e-harness` SDK-direct tests pass after extraction
- [x] Touched-Rust size policy passes on the issue branch

# Files to touch

- `specs/6699-split-sdk-direct-live-probe-tranche-1.md`
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/sdk_direct/**`
- `crates/kamn-e2e-harness/tests/**` or `crates/kamn-e2e-harness/src/drivers/**` contract coverage needed for the tranche

# Error semantics

- Contract tests fail hard with exact missing-path, file-size, and staged-root-cap details
- Extraction keeps the current hard-fail error behavior for live probe implementations and validators
- No fallback to inline production probe definitions or alternate wiring layouts

# Test plan

1. Add a red contract that asserts the new S-01 through S-05 production tranche layout and the staged root cap.
2. Extract the first production tranche into bounded sibling modules until the contract passes.
3. Run focused `kamn-e2e-harness` tests covering SDK-direct driver behavior and command/runtime contracts.
4. Run touched-Rust size policy on the full issue write set.

# Evidence

- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs` reduced from `1645` LOC to `1289` LOC.
- Extracted production tranche files:
  - `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_one.rs` (`90` LOC)
  - `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_one/discovery_direct_message_probes.rs` (`104` LOC)
  - `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_one/channel_task_probes.rs` (`181` LOC)
  - `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_one/escrow_probe_support.rs` (`55` LOC)
- Clean verification was run from detached worktree `/tmp/kamn-6699-verify-head4-tI0CQ7` to avoid unrelated background edits in the primary issue worktree.
- Verification commands:
  - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6699-clean-touched-size-final.json`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-e2e-harness sdk_direct -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-e2e-harness --test sdk_direct_tranche_one_extraction_contract -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-e2e-harness --test command_contract -- --nocapture`

# Deviations

- No functional deviations from the original issue scope.
- During refactor, the direct-message probe body needed one additional helper split to satisfy the touched-Rust function-size policy while preserving behavior.
