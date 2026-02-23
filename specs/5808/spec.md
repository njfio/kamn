# Spec: Issue #5808 - Activate Opt-In Live S-02 Direct-Message Scenario Across E2E Drivers

- Issue: #5808
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
`docs/review/gaps-and-issues-r54.md` calls out next-cycle scenario activation beyond `S-01/S-04/S-06`. The harness currently binds live driver execution only for those three scenarios, leaving `S-02` (Direct Message Round-Trip) on scaffold-only behavior even when live execution toggles are enabled.

## Scope
In scope:
- Add opt-in live `S-02` execution path in all three drivers:
  - `sdk-direct` (`kamn-agent-lib` send/query operations)
  - `cli-scripted` (`kamn-cli` send-message/query-message commands)
  - `mcp-agent` (`kamn-mcp-server` `send_message`/`query_message` tool calls)
- Preserve fail-closed semantics: `S-02` returns `fail` when live probe execution errors.
- Add conformance tests for `S-02` execution mapping and failure behavior.
- Update lifecycle/milestone metadata for this delivery slice.

Out of scope:
- Service API schema or wire-format changes.
- Activation of scenarios beyond `S-02`.
- Shell/workflow/template surface changes.

## Acceptance Criteria
- AC-1: Given `KAMN_E2E_SDK_DIRECT_LIVE` is truthy, when `SdkDirectDriver::execute("S-02")` runs, then the dedicated live S-02 probe executes and `status` is `fail` on probe error.
- AC-2: Given `KAMN_E2E_CLI_SCRIPTED_LIVE` is truthy, when `CliScriptedDriver::execute("S-02")` runs, then the dedicated live S-02 runner executes and `status` is `fail` on runner error.
- AC-3: Given `KAMN_E2E_MCP_AGENT_LIVE` is truthy, when `McpAgentDriver::execute("S-02")` runs, then the dedicated live S-02 probe executes and `status` is `fail` on probe error.
- AC-4: Given live mode disabled (or scenarios outside live-bound set), when drivers execute scenarios, then existing deterministic behavior remains unchanged.
- AC-5: Issue lifecycle artifacts and milestone index are updated to reflect this scenario activation slice.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `SdkDirectDriver::execute("S-02")` with live probe error | Driver returns `status="fail"`. |
| C-02 | AC-2 | Functional | `CliScriptedDriver::execute("S-02")` with live runner error | Driver returns `status="fail"`. |
| C-03 | AC-3 | Functional | `McpAgentDriver::execute("S-02")` with live probe error | Driver returns `status="fail"`. |
| C-04 | AC-4 | Regression | Existing `S-01/S-04/S-06` and disabled-live paths | Existing behavior remains passing/unchanged. |
| C-05 | AC-5 | Conformance | Milestone index + lifecycle artifact inspection | `#5808` appears with active/completed lifecycle updates. |

## Test Mapping
- `cargo test -p kamn-e2e-harness spec_c03_live_s02_driver_path_fails_closed_when_message_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness spec_c11_live_s02_driver_path_fails_closed_when_message_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness --test sdk_direct_live_toggle_contract -- --nocapture`
- `cargo test -p kamn-e2e-harness --test cli_scripted_live_toggle_contract -- --nocapture`
- `cargo test -p kamn-e2e-harness --test mcp_agent_live_toggle_contract -- --nocapture`
- `cargo test -p kamn-e2e-harness -- --nocapture`

## Success Metrics / Observable Signals
- `S-02` is included in live-bound scenario routing for all three drivers.
- New conformance tests for `S-02` fail-closed behavior pass.
- Existing live-scenario and non-live regressions remain green.
