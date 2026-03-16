# 7003 Live Task-Lifecycle MCP Parity

## Objective
Prove one bounded MCP-agent parity slice for the existing live S-04 task-lifecycle lane on current `main`. The proof must be anchored to one explicit local-heavy ignored integration test that executes the checked-in MCP-agent S-04 live probe against a running local Kolme runtime and local KAMN API runtime. Publish one operator-facing validation runbook and one hard-fail docs contract that capture the exact commands and the exact limits of the claim.

## Inputs/Outputs
- Inputs:
  - running local Kolme API runtime reachable through `KAMN_KOLME_ENDPOINT` when required by the operator setup
  - running local KAMN API node reachable through `KAMN_ENDPOINT`
  - built `kamn-mcp-server` binary at a deterministic path via `KAMN_E2E_MCP_AGENT_BINARY`
  - live MCP env gate `KAMN_E2E_MCP_AGENT_LIVE=true`
  - live agent name via `KAMN_AGENT_NAME`
  - MCP key-file path via `KAMN_AGENT_KEY_FILE`
- Outputs:
  - one passing ignored integration test for MCP-agent S-04 parity
  - one validation runbook under `docs/validation/`
  - one hard-fail docs contract under `crates/kamn-node/tests/`
  - updated runtime-proof index entry linking the runbook

## Boundaries/Non-goals
- Do not claim CLI parity in this issue.
- Do not claim crash recovery.
- Do not claim Solana-backed settlement or bridge settlement.
- Do not add a new harness scenario, new runtime mode, or new MCP tool surface.
- Do not weaken or replace the existing `sdk-direct` live S-04 proof.
- Do not rely on the harness `run` scaffold alone as the proof anchor.

## Failure Modes
- required live env vars are missing or empty
- `kamn-mcp-server` binary path is missing or not executable
- MCP key-file path is missing or unusable
- local Kolme runtime is unavailable for the declared setup
- local KAMN API endpoint is unavailable
- MCP `probe-create-task` output is missing `task_id`
- MCP `probe-fund-escrow` output is missing `escrow_id` or `state`
- MCP `probe-accept-task` or `probe-complete-task` output is missing `state`
- MCP `probe-release-escrow` output is missing `state`
- release response returns mismatched escrow identity or non-released state
- runbook overstates the proof beyond MCP parity on local runtime
- runtime-proof index omits the new runbook

## Acceptance Criteria (testable booleans)
- [ ] `crates/kamn-e2e-harness/tests/live_s04_mcp_agent_execution.rs` exists and executes the real MCP-agent S-04 probe when invoked explicitly with `--ignored`.
- [ ] `cargo test -p kamn-node --test live_task_lifecycle_mcp_parity_slice_contract -- --nocapture` passes.
- [ ] `docs/validation/live-task-lifecycle-mcp-parity-slice.md` exists and states the proof is bounded to local-heavy MCP-agent S-04 parity on current `main`.
- [ ] The runbook contains the exact ignored test command and required live env names, including `KAMN_E2E_MCP_AGENT_LIVE`, `KAMN_E2E_MCP_AGENT_BINARY`, `KAMN_ENDPOINT`, `KAMN_KOLME_ENDPOINT`, `KAMN_AGENT_NAME`, and `KAMN_AGENT_KEY_FILE`.
- [ ] The runbook explicitly states that the slice does not prove crash recovery, Solana-backed settlement, bridge settlement, CLI parity, or production readiness.
- [ ] `docs/validation/current-proven-runtime-slices.md` links the MCP parity runbook.
- [ ] The existing `sdk-direct` live task-lifecycle proof remains intact and unchanged in meaning.

## Files to Touch
- `specs/7003-live-task-lifecycle-mcp-parity.md`
- `docs/validation/live-task-lifecycle-mcp-parity-slice.md`
- `docs/validation/current-proven-runtime-slices.md`
- `crates/kamn-node/tests/live_task_lifecycle_mcp_parity_slice_contract.rs`
- `crates/kamn-e2e-harness/tests/live_s04_mcp_agent_execution.rs`
- optionally `docs/review/corrected-audit-response-2026-03-14.md` if the proof-index wording needs to mention the new slice

## Error Semantics
- The explicit ignored integration test must fail hard when required live env vars are missing or empty.
- The explicit ignored integration test must fail hard when MCP output omits required fields or returns invalid state.
- The docs contract must fail hard when the runbook or proof index omits required markers or overstates the claim.
- No silent fallback from MCP-agent live execution to another driver is allowed.

## Test Plan
1. Red:
   - add docs contract asserting the new runbook and proof-index markers; confirm it fails because the doc is absent
2. Green:
   - publish the runbook
   - add the ignored integration proof test that invokes `McpAgentDriver::from_env(ExecutionMode::McpTau)?.execute("S-04")`
   - wire the runtime-proof index entry
3. Verification:
   - `cargo test -p kamn-node --test live_task_lifecycle_mcp_parity_slice_contract -- --nocapture`
   - `cargo test -p kamn-e2e-harness --test live_s04_mcp_agent_execution -- --ignored --exact --nocapture`
   - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/7003-touched-size.json`
4. Local live evidence:
   - build `kamn-node`, `kamn-mcp-server`, and `kamn-e2e-harness`
   - run the ignored MCP-agent S-04 proof test against local Kolme and KAMN API runtime

## Notes
- The proof anchor is the explicit ignored integration test, not the harness scaffold alone.
- This issue is parity-only for the existing bounded local-heavy task-lifecycle slice.

## Final Evidence
- `cargo test -p kamn-node --test live_task_lifecycle_mcp_parity_slice_contract -- --nocapture`
- `cargo test -p kamn-e2e-harness --test live_s04_mcp_agent_execution --no-run`
- `cargo build -p kamn-node -p kamn-mcp-server -p kamn-e2e-harness`
- `KAMN_E2E_MCP_AGENT_LIVE=true KAMN_E2E_MCP_AGENT_BINARY=/home/n/Code/kamn/target/debug/kamn-mcp-server KAMN_ENDPOINT=http://127.0.0.1:18180 KAMN_KOLME_ENDPOINT=http://127.0.0.1:13100 KAMN_AGENT_NAME=kamn-live-mcp-s04-proof KAMN_AGENT_KEY_FILE=/tmp/kamn-live-mcp-s04-proof.key cargo test -p kamn-e2e-harness --test live_s04_mcp_agent_execution integration_live_s04_mcp_agent_task_lifecycle_probe_against_local_runtime -- --ignored --exact --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/7003-touched-size.json`
- touched-Rust result: `policy_decision=GO`

## Deviations
- None. The proof remained bounded to MCP parity for the existing local-heavy S-04 task-lifecycle lane and did not claim CLI parity, crash recovery, or external settlement.
