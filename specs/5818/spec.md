# Spec: Issue #5818 - Activate Opt-In Live S-05 Escrow-Settlement Scenario Across E2E Drivers

- Issue: #5818
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
The e2e harness currently routes live probes for `S-01`, `S-02`, `S-03`, `S-04`, and `S-06`, but `S-05` remains scaffold-only when live toggles are enabled. This leaves a P0 escrow-settlement scenario outside live-bound driver contracts.

## Scope
In scope:
- Add live `S-05` routing and probe/runner support to:
  - `sdk-direct`
  - `cli-scripted`
  - `mcp-agent`
- Preserve fail-closed semantics: any live `S-05` probe failure maps scenario status to `fail`.
- Add conformance tests first (RED) proving `execute("S-05")` fail-closed behavior.
- Update milestone and lifecycle artifacts.

Out of scope:
- Protocol/schema/wire-format changes.
- Activation of scenarios beyond `S-05`.
- Shell/workflow/template changes.

## Acceptance Criteria
- AC-1: Given `KAMN_E2E_SDK_DIRECT_LIVE` is truthy, `SdkDirectDriver::execute("S-05")` runs a dedicated live S-05 probe and returns `fail` on probe error.
- AC-2: Given `KAMN_E2E_CLI_SCRIPTED_LIVE` is truthy, `CliScriptedDriver::execute("S-05")` runs a dedicated live S-05 runner and returns `fail` on runner error.
- AC-3: Given `KAMN_E2E_MCP_AGENT_LIVE` is truthy, `McpAgentDriver::execute("S-05")` runs a dedicated live S-05 probe and returns `fail` on probe error.
- AC-4: Existing live-bound scenarios (`S-01/S-02/S-03/S-04/S-06`) and non-live scenarios remain regression-green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `SdkDirectDriver::execute("S-05")` with live probe error | Returns `status="fail"`. |
| C-02 | AC-2 | Functional | `CliScriptedDriver::execute("S-05")` with live runner error | Returns `status="fail"`. |
| C-03 | AC-3 | Functional | `McpAgentDriver::execute("S-05")` with live probe error | Returns `status="fail"`. |
| C-04 | AC-4 | Regression | `cargo test -p kamn-e2e-harness -- --nocapture` | Existing scenarios/toggles remain green. |
| C-05 | AC-4 | Conformance | milestone index + lifecycle artifacts | Slice markers/issue refs align with completed state. |

## Test Mapping
- `cargo test -p kamn-e2e-harness drivers::sdk_direct::tests::spec_c05_live_s05_driver_path_fails_closed_when_escrow_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness drivers::cli_scripted::tests::spec_c05_live_s05_driver_path_fails_closed_when_escrow_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness drivers::mcp_agent::tests::spec_c13_live_s05_driver_path_fails_closed_when_escrow_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness -- --nocapture`

## Success Metrics / Observable Signals
- `S-05` appears in live-bound routing for all three drivers.
- New `S-05` fail-closed conformance tests pass.
- Regression suites remain green.
