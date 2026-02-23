# Spec: Issue #5830 - Activate Opt-In Live S-12 Retention/Deletion Scenario Across E2E Drivers

- Issue: #5830
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
Live driver activation currently covers scenarios through `S-11`. `S-12` (`Content Retention & Deletion`) remains scaffold-only, so retention-expiry, tombstone/redaction, and lifecycle-query continuity are not exercised in live-bound conformance execution.

## Scope
In scope:
- Add live `S-12` routing/probe support in:
  - `sdk-direct`
  - `cli-scripted`
  - `mcp-agent`
- Implement deterministic `S-12` probe flow that validates:
  - content registration succeeds,
  - lifecycle expiry transition succeeds,
  - tombstone/redaction transition succeeds with explicit marker,
  - lifecycle query remains coherent after tombstone.
- Add minimal cross-surface operation support needed for `S-12` probes:
  - service API endpoint payload/auth scope mapping,
  - SDK client methods/types,
  - agent-lib facade methods,
  - CLI commands,
  - MCP tools/dispatch.
- Add RED fail-closed conformance tests first for `execute("S-12")` in all three drivers.

Out of scope:
- New dependencies.
- Protocol framing changes.
- Scenario activation beyond `S-12`.

## Acceptance Criteria
- AC-1: Given `KAMN_E2E_SDK_DIRECT_LIVE` is truthy, `SdkDirectDriver::execute("S-12")` runs a dedicated retention/deletion probe and returns `fail` when `S-12` validation errors.
- AC-2: Given `KAMN_E2E_CLI_SCRIPTED_LIVE` is truthy, `CliScriptedDriver::execute("S-12")` runs a dedicated retention/deletion probe and returns `fail` when `S-12` validation errors.
- AC-3: Given `KAMN_E2E_MCP_AGENT_LIVE` is truthy, `McpAgentDriver::execute("S-12")` runs a dedicated retention/deletion probe and returns `fail` when `S-12` validation errors.
- AC-4: Service/API/CLI/MCP surfaces expose deterministic S-12 operation contracts used by probes (register/expire/tombstone/query) with coherent scope-policy and inventory coverage.
- AC-5: Existing live-bound scenarios (`S-01`..`S-11`) and non-live scenario behavior remain regression-green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `SdkDirectDriver::execute("S-12")` with S-12 probe error | `status="fail"` |
| C-02 | AC-2 | Functional | `CliScriptedDriver::execute("S-12")` with S-12 probe error | `status="fail"` |
| C-03 | AC-3 | Functional | `McpAgentDriver::execute("S-12")` with S-12 probe error | `status="fail"` |
| C-04 | AC-4 | Integration | command/tool/route scope contracts for S-12 operations | deterministic pass + schema/status markers |
| C-05 | AC-5 | Regression | `cargo test -p kamn-e2e-harness -- --nocapture` | existing scenario/toggle contracts remain green |
| C-06 | AC-5 | Conformance | `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture` | spec-volume/docs guardrails remain green |

## Test Mapping
- `cargo test -p kamn-e2e-harness live_s12_driver_path_fails_closed_when_retention_deletion_probe_errors -- --nocapture`
- `cargo test -p kamn-cli subcommand_surface_contract command_activation_contract -- --nocapture`
- `cargo test -p kamn-mcp-server tool_inventory_contract tool_dispatch_contract stdio_protocol_contract -- --nocapture`
- `cargo test -p kamn-node service_api_endpoint_tests -- --nocapture`
- `cargo test -p kamn-e2e-harness -- --nocapture`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics / Observable Signals
- `S-12` is live-bound in all three driver routing maps.
- `S-12` probes verify deterministic register/expire/tombstone/query continuity and fail closed on any validation drift.
- CLI and MCP inventories include explicit S-12 operations.
- Conformance/regression/docs-contract/mutation/workspace gates are green.
