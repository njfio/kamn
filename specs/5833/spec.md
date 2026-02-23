# Spec: Issue #5833 - Activate Opt-In Live S-13 Bridge-Forwarding Scenario Across E2E Drivers

- Issue: #5833
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
Live driver activation currently covers scenarios through `S-12`. `S-13` (`Bridge Message Forwarding`) remains scaffold-only, so deterministic bridge submit/forward/query continuity is not exercised in live-bound conformance execution.

## Scope
In scope:
- Add live `S-13` routing/probe support in:
  - `sdk-direct`
  - `cli-scripted`
  - `mcp-agent`
- Implement deterministic `S-13` probe flow that validates:
  - bridge message submission succeeds,
  - bridge forward transition succeeds with deterministic bridge and target markers,
  - bridge query projection remains coherent with forward result.
- Add minimal cross-surface operation support needed for `S-13` probes:
  - service API endpoint payload/auth scope mapping,
  - SDK client methods/types,
  - agent-lib facade methods,
  - CLI commands,
  - MCP tools/dispatch.
- Add RED fail-closed conformance tests first for `execute("S-13")` in all three drivers.

Out of scope:
- New dependencies.
- Protocol framing changes.
- Scenario activation beyond `S-13`.

## Acceptance Criteria
- AC-1: Given `KAMN_E2E_SDK_DIRECT_LIVE` is truthy, `SdkDirectDriver::execute("S-13")` runs a dedicated bridge-forwarding probe and returns `fail` when `S-13` validation errors.
- AC-2: Given `KAMN_E2E_CLI_SCRIPTED_LIVE` is truthy, `CliScriptedDriver::execute("S-13")` runs a dedicated bridge-forwarding probe and returns `fail` when `S-13` validation errors.
- AC-3: Given `KAMN_E2E_MCP_AGENT_LIVE` is truthy, `McpAgentDriver::execute("S-13")` runs a dedicated bridge-forwarding probe and returns `fail` when `S-13` validation errors.
- AC-4: Service/API/CLI/MCP surfaces expose deterministic S-13 operation contracts used by probes (`submit_bridge_message` / `forward_bridge_message` / `query_bridge_message`) with coherent scope-policy and inventory coverage.
- AC-5: Existing live-bound scenarios (`S-01`..`S-12`) and non-live scenario behavior remain regression-green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `SdkDirectDriver::execute("S-13")` with S-13 probe error | `status="fail"` |
| C-02 | AC-2 | Functional | `CliScriptedDriver::execute("S-13")` with S-13 probe error | `status="fail"` |
| C-03 | AC-3 | Functional | `McpAgentDriver::execute("S-13")` with S-13 probe error | `status="fail"` |
| C-04 | AC-4 | Integration | command/tool/route scope contracts for S-13 operations | deterministic pass + schema/status markers |
| C-05 | AC-5 | Regression | `cargo test -p kamn-e2e-harness -- --nocapture` | existing scenario/toggle contracts remain green |
| C-06 | AC-5 | Conformance | docs-contract review lanes | spec-volume/docs guardrails remain green |

## Test Mapping
- `cargo test -p kamn-e2e-harness live_s13_driver_path_fails_closed_when_bridge_forwarding_probe_errors -- --nocapture`
- `cargo test -p kamn-cli subcommand_surface_contract command_activation_contract -- --nocapture`
- `cargo test -p kamn-mcp-server tool_inventory_contract tool_dispatch_contract stdio_protocol_contract -- --nocapture`
- `cargo test -p kamn-node service_api_endpoint_tests -- --nocapture`
- `cargo test -p kamn-e2e-harness -- --nocapture`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics / Observable Signals
- `S-13` is live-bound in all three driver routing maps.
- `S-13` probes verify deterministic submit/forward/query continuity and fail closed on validation drift.
- CLI and MCP inventories include explicit S-13 operations.
- Conformance/regression/docs-contract/mutation/workspace gates are green.
