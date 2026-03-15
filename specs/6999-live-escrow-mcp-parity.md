# 6999 Live Escrow MCP Parity

## Objective
Prove one bounded MCP-agent parity slice for the existing live S-05 escrow-settlement lane on current `main`. The proof must be anchored to one explicit local-heavy ignored integration test that executes the checked-in MCP-agent S-05 live probe against a running local Kolme runtime and local KAMN API runtime. Publish one operator-facing validation runbook and one hard-fail docs contract that capture the exact commands and the exact limits of the claim.

## Inputs/Outputs
- Inputs:
  - running local Kolme API runtime reachable through `KAMN_KOLME_ENDPOINT` when needed by the operator setup
  - running local KAMN API node reachable through `KAMN_ENDPOINT`
  - built `kamn-mcp-server` binary at a deterministic path via `KAMN_E2E_MCP_AGENT_BINARY`
  - live MCP env gate `KAMN_E2E_MCP_AGENT_LIVE=true`
  - live agent name via `KAMN_AGENT_NAME`
  - MCP key-file path via `KAMN_AGENT_KEY_FILE`
- Outputs:
  - one passing ignored integration test for MCP-agent S-05 parity
  - one validation runbook under `docs/validation/`
  - one hard-fail docs contract under `crates/kamn-node/tests/`
  - updated runtime-proof index entry linking the runbook

## Boundaries/Non-goals
- Do not claim Solana-backed settlement.
- Do not claim bridge settlement.
- Do not claim external-chain settlement.
- Do not add new MCP tools, new probe protocol surface, or new CI/workflow surface.
- Do not weaken or replace the existing `sdk-direct` or CLI-scripted proof slices.
- Do not rely on the harness `run` scaffold alone as the proof anchor.

## Failure Modes
- required live env vars are missing or empty
- `kamn-mcp-server` binary path is missing or not executable
- MCP key-file path is missing or unusable
- local Kolme runtime is unavailable for the declared setup
- local KAMN API endpoint is unavailable
- MCP `probe-fund-escrow` output is missing `escrow_id` or `state`
- MCP `probe-release-escrow` output is missing `escrow_id` or `state`
- release response returns mismatched escrow identity or non-released state
- runbook overstates the proof beyond MCP parity on local runtime
- runtime-proof index omits the new runbook

## Acceptance Criteria (testable booleans)
- [ ] `crates/kamn-e2e-harness/tests/live_s05_mcp_agent_execution.rs` exists and executes the real MCP-agent S-05 probe when invoked explicitly with `--ignored`.
- [ ] `cargo test -p kamn-node --test live_escrow_mcp_parity_slice_contract -- --nocapture` passes.
- [ ] `docs/validation/live-escrow-mcp-parity-slice.md` exists and states the proof is bounded to local-heavy MCP-agent S-05 parity on current `main`.
- [ ] The runbook contains the exact ignored test command and required live env names, including `KAMN_E2E_MCP_AGENT_LIVE`, `KAMN_E2E_MCP_AGENT_BINARY`, and `KAMN_AGENT_KEY_FILE`.
- [ ] The runbook explicitly states that the slice does not prove Solana-backed settlement, bridge settlement, or external-chain settlement.
- [ ] `docs/validation/current-proven-runtime-slices.md` links the MCP parity runbook.
- [ ] The existing `sdk-direct` and CLI-scripted live escrow settlement proofs remain intact and unchanged in meaning.

## Files to Touch
- `specs/6999-live-escrow-mcp-parity.md`
- `docs/validation/live-escrow-mcp-parity-slice.md`
- `docs/validation/current-proven-runtime-slices.md`
- `crates/kamn-node/tests/live_escrow_mcp_parity_slice_contract.rs`
- `crates/kamn-e2e-harness/tests/live_s05_mcp_agent_execution.rs`
- optionally `docs/review/corrected-audit-response-2026-03-14.md` if the proof index wording needs to mention the new slice

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
   - add the ignored integration proof test that invokes `McpAgentDriver::from_env(ExecutionMode::McpTau)?.execute("S-05")`
   - wire the runtime-proof index entry
3. Verification:
   - `cargo test -p kamn-node --test live_escrow_mcp_parity_slice_contract -- --nocapture`
   - `cargo test -p kamn-e2e-harness --test live_s05_mcp_agent_execution -- --ignored --exact --nocapture`
   - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6999-touched-size.json`
4. Local live evidence:
   - build `kamn-node`, `kamn-mcp-server`, and `kamn-e2e-harness`
   - run the ignored MCP-agent S-05 proof test against local Kolme and KAMN API runtime

## Notes
- The proof anchor is the explicit ignored integration test, not the harness scaffold alone.
- This issue is parity-only for the existing bounded local-heavy escrow slice.

## Final Evidence
- `cargo test -p kamn-node --test live_escrow_mcp_parity_slice_contract -- --nocapture`
- `cargo test -p kamn-mcp-server --bin kamn-mcp-server regression_issue_6197_load_signing_key_from_file_consumes_key_material -- --nocapture`
- `cargo build -p kamn-mcp-server`
- `KAMN_E2E_MCP_AGENT_LIVE=true KAMN_E2E_MCP_AGENT_BINARY=/home/n/Code/kamn/target/debug/kamn-mcp-server KAMN_ENDPOINT=http://127.0.0.1:18182 KAMN_KOLME_ENDPOINT=http://127.0.0.1:13100 KAMN_AGENT_NAME=kamn-live-mcp-s05-proof KAMN_AGENT_KEY_FILE=/tmp/kamn-live-mcp-s05-proof.key cargo test -p kamn-e2e-harness --test live_s05_mcp_agent_execution integration_live_s05_mcp_agent_escrow_settlement_probe_against_local_runtime -- --ignored --exact --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6999-touched-size.json`
- touched-Rust result: `policy_decision=GO`

## Deviations
- Phase 6 exposed a real MCP production defect before the live proof could go green.
- `kamn-mcp-server` had been constructing legacy `kamn:did:agent:<name>` identities from key-file input, which current service-auth policy rejects in production mode.
- The fix was kept bounded to the issue scope:
  - `kamn-mcp-server` now derives a self-certifying DID bound to the signing key from `KAMN_AGENT_KEY_FILE`
  - the explicit live S-05 MCP proof test now materializes matching key material into the configured key-file path before launching the MCP server
