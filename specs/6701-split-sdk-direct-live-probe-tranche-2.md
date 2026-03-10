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

- [x] S-06 through S-10 live probe implementations are extracted from `sdk_direct.rs` into sibling production modules
- [x] Any tranche support files touched by the extraction remain <= 200 LOC
- [x] `sdk_direct.rs` preserves real production wiring and delegates to the extracted tranche
- [x] `sdk_direct.rs` is reduced to <= 1000 LOC after the tranche extraction
- [x] A contract test fails if the tranche layout regresses or the staged root cap is exceeded
- [x] Relevant `kamn-e2e-harness` SDK-direct tests pass after extraction
- [x] Touched-Rust size policy passes on the issue branch

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

# Evidence

- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs` reduced from `1289` LOC to `841` LOC.
- Extracted production tranche files:
  - `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_two.rs` (`79` LOC)
  - `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_two/message_query_support.rs` (`85` LOC)
  - `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_two/proof_replay_probes.rs` (`132` LOC)
  - `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_two/recovery_failover_probes.rs` (`77` LOC)
  - `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_two/recovery_failover_probes/crash_recovery_probe.rs` (`89` LOC)
  - `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_two/recovery_failover_probes/transport_failover_probe.rs` (`102` LOC)
  - `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_two/topology_coherence_probe.rs` (`89` LOC)
- Clean verification was run from detached worktree `/tmp/kamn-6701-verify2-ARe9oD` to avoid unrelated background edits in the shared issue worktree.
- Verification commands:
  - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6701-clean-touched-size-final.json`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-e2e-harness --test sdk_direct_tranche_two_extraction_contract -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-e2e-harness sdk_direct -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-e2e-harness --test command_contract -- --nocapture`

# Deviations

- No functional deviations from the issue scope.
- The recovery/failover portion needed one additional nested split under `recovery_failover_probes/` to satisfy the touched-Rust function-size policy on the clean verification head.
