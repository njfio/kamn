# 7001 Live Task Lifecycle Slice

## Objective
Prove one bounded live task-lifecycle slice on current `main` by executing the existing `kamn-e2e-harness` `sdk-direct` S-04 task-lifecycle probe against a real local Kolme runtime and local KAMN API runtime. Publish one operator-facing validation runbook and one hard-fail docs contract that capture the exact command, exact env, and exact limits of the claim.

## Inputs/Outputs
- Inputs:
  - running local KAMN API runtime reachable through `KAMN_ENDPOINT`
  - running local Kolme API runtime reachable through `KAMN_KOLME_ENDPOINT`
  - live env gate `KAMN_E2E_SDK_DIRECT_LIVE=true`
  - live agent name via `KAMN_AGENT_NAME`
- Outputs:
  - one passing ignored integration test for `sdk-direct` S-04 task lifecycle
  - one validation runbook under `docs/validation/`
  - one hard-fail docs contract under `crates/kamn-node/tests/`
  - one runtime-proof index entry linking the new runbook

## Boundaries/Non-goals
- Do not claim MCP or CLI parity in this issue.
- Do not claim crash recovery.
- Do not claim Solana-backed settlement or bridge settlement.
- Do not add a new harness scenario or new runtime mode.
- Do not weaken the existing service-api vertical slice proof.

## Failure Modes
- required live env vars are missing or empty
- local KAMN API endpoint is unavailable
- local Kolme runtime is unavailable
- create-task response is missing `task_id`
- fund-escrow response is missing `escrow_id`
- accept-task or complete-task response is missing `state`
- release-escrow response is missing `state`
- runbook overstates the proof beyond one bounded local-heavy `sdk-direct` S-04 lane
- runtime-proof index omits the new slice

## Acceptance Criteria (testable booleans)
- [ ] `crates/kamn-e2e-harness/tests/live_s04_sdk_direct_execution.rs` exists and executes the real `sdk-direct` S-04 probe when invoked explicitly with `--ignored`.
- [ ] `cargo test -p kamn-node --test live_task_lifecycle_slice_contract -- --nocapture` passes.
- [ ] `docs/validation/live-task-lifecycle-slice.md` exists and states the proof is bounded to one local-heavy `sdk-direct` S-04 lane on current `main`.
- [ ] The runbook contains the exact ignored test command and required live env names, including `KAMN_E2E_SDK_DIRECT_LIVE`, `KAMN_ENDPOINT`, `KAMN_KOLME_ENDPOINT`, and `KAMN_AGENT_NAME`.
- [ ] The runbook explicitly states that the slice does not prove crash recovery, Solana-backed settlement, bridge settlement, or production readiness.
- [ ] `docs/validation/current-proven-runtime-slices.md` links the new runbook.

## Files to Touch
- `specs/7001-live-task-lifecycle-slice.md`
- `docs/validation/live-task-lifecycle-slice.md`
- `docs/validation/current-proven-runtime-slices.md`
- `crates/kamn-node/tests/live_task_lifecycle_slice_contract.rs`
- `crates/kamn-e2e-harness/tests/live_s04_sdk_direct_execution.rs`
- optionally `docs/review/corrected-audit-response-2026-03-14.md` if the proof index wording needs to mention the new slice

## Error Semantics
- The explicit ignored integration test must fail hard when required live env vars are missing or empty.
- The explicit ignored integration test must fail hard when live task-lifecycle probe output is missing required identifiers or states.
- The docs contract must fail hard when the runbook or proof index omits required markers or overstates the claim.
- No silent fallback from `sdk-direct` live execution to another driver is allowed.

## Test Plan
1. Red:
   - add docs contract asserting the new runbook and proof-index markers; confirm it fails because the doc is absent
2. Green:
   - publish the runbook
   - add the ignored integration proof test that invokes `SdkDirectDriver::from_env().execute("S-04")`
   - wire the runtime-proof index entry
3. Verification:
   - `cargo test -p kamn-node --test live_task_lifecycle_slice_contract -- --nocapture`
   - `cargo test -p kamn-e2e-harness --test live_s04_sdk_direct_execution -- --ignored --exact --nocapture`
   - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/7001-touched-size.json`
4. Local live evidence:
   - build `kamn-node` and `kamn-e2e-harness`
   - run the ignored `sdk-direct` S-04 proof test against local Kolme and local KAMN API runtime

## Notes
- The proof anchor is the explicit ignored integration test, not the harness scaffold alone.
- This issue is intentionally bounded to one `sdk-direct` live task-lifecycle lane.
