# 6997 Live Escrow CLI Parity

## Objective
Prove one bounded CLI-scripted parity slice for the existing live S-05 escrow-settlement lane on current `main`. The proof must be anchored to one explicit local-heavy ignored integration test that executes the checked-in CLI-scripted S-05 live probe against a running local Kolme runtime and local KAMN API node(s). Publish one operator-facing validation runbook and one hard-fail docs contract that capture the exact commands and the exact limits of the claim.

## Inputs/Outputs
- Inputs:
  - running local Kolme API runtime reachable through `KAMN_KOLME_ENDPOINT`
  - running local KAMN API node reachable through `KAMN_ENDPOINT`
  - built `kamn-cli` binary at a deterministic path
  - live CLI env gate `KAMN_E2E_CLI_SCRIPTED_LIVE=true`
  - live agent name via `KAMN_AGENT_NAME`
- Outputs:
  - one passing ignored integration test for CLI-scripted S-05 parity
  - one validation runbook under `docs/validation/`
  - one hard-fail docs contract under `crates/kamn-node/tests/`
  - updated runtime-proof index entry linking the runbook

## Boundaries/Non-goals
- Do not claim Solana-backed settlement.
- Do not claim bridge settlement.
- Do not claim external-chain settlement.
- Do not add new runtime drivers, new CLI commands, or new CI/workflow surface.
- Do not weaken or replace the existing `sdk-direct` proof from `#6995`.
- Do not rely on the harness `run` scaffold alone as the proof anchor.

## Failure Modes
- required live env vars are missing or empty
- `kamn-cli` binary path is missing or not executable
- local Kolme runtime is unavailable
- local KAMN API endpoint is unavailable
- CLI `fund-escrow` output is missing `escrow_id` or `state`
- CLI `release-escrow` output is missing `escrow_id` or `state`
- release response returns mismatched escrow identity or non-released state
- runbook overstates the proof beyond CLI parity on local runtime
- runtime-proof index omits the new runbook

## Acceptance Criteria (testable booleans)
- [ ] `crates/kamn-e2e-harness/tests/live_s05_cli_scripted_execution.rs` exists and executes the real CLI-scripted S-05 probe when invoked explicitly with `--ignored`.
- [ ] `cargo test -p kamn-node --test live_escrow_cli_parity_slice_contract -- --nocapture` passes.
- [ ] `docs/validation/live-escrow-cli-parity-slice.md` exists and states the proof is bounded to local-heavy CLI-scripted S-05 parity on current `main`.
- [ ] The runbook contains the exact ignored test command and required live env names, including `KAMN_E2E_CLI_SCRIPTED_LIVE` and `KAMN_E2E_CLI_BINARY`.
- [ ] The runbook explicitly states that the slice does not prove Solana-backed settlement, bridge settlement, or external-chain settlement.
- [ ] `docs/validation/current-proven-runtime-slices.md` links the CLI parity runbook.
- [ ] The existing `sdk-direct` live escrow settlement proof remains intact and unchanged in meaning.

## Files to Touch
- `specs/6997-live-escrow-cli-parity.md`
- `docs/validation/live-escrow-cli-parity-slice.md`
- `docs/validation/current-proven-runtime-slices.md`
- `crates/kamn-node/tests/live_escrow_cli_parity_slice_contract.rs`
- `crates/kamn-e2e-harness/tests/live_s05_cli_scripted_execution.rs`
- optionally `docs/review/corrected-audit-response-2026-03-14.md` if the proof index wording needs to mention the new slice

## Error Semantics
- The explicit ignored integration test must fail hard when required live env vars are missing or empty.
- The explicit ignored integration test must fail hard when CLI output omits required fields or returns invalid state.
- The docs contract must fail hard when the runbook or proof index omits required markers or overstates the claim.
- No silent fallback from CLI-scripted live execution to another driver is allowed.

## Test Plan
1. Red:
   - add docs contract asserting the new runbook and proof-index markers; confirm it fails because the doc is absent
2. Green:
   - publish the runbook
   - add the ignored integration proof test that invokes `CliScriptedDriver::from_env().execute("S-05")`
   - wire the runtime-proof index entry
3. Verification:
   - `cargo test -p kamn-node --test live_escrow_cli_parity_slice_contract -- --nocapture`
   - `cargo test -p kamn-e2e-harness --test live_s05_cli_scripted_execution -- --ignored --exact --nocapture`
   - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6997-touched-size.json`
4. Local live evidence:
   - build `kamn-node`, `kamn-cli`, and `kamn-e2e-harness`
   - run the ignored CLI-scripted S-05 proof test against local Kolme and KAMN API runtime

## Notes
- The proof anchor is the explicit ignored integration test, not the harness scaffold alone.
- This issue is parity-only for the existing bounded local-heavy escrow slice.
