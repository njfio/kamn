# Spec: Issue #5824 - Activate Opt-In Live S-09 Transport-Failover Scenario Across E2E Drivers

- Issue: #5824
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
The harness supports live probes through `S-08`, but `S-09` (Transport Failover) is still scaffold-only in live driver routing. This leaves failover-bound continuity checks outside live-bound conformance execution.

## Scope
In scope:
- Add live `S-09` routing/probe support in:
  - `sdk-direct`
  - `cli-scripted`
  - `mcp-agent`
- Implement deterministic `S-09` failover probes that execute:
  - pre-failover send/query validation through primary endpoint,
  - failover-boundary health probe validation through failover endpoint,
  - post-failover send/query validation through failover endpoint,
  - fail-closed continuity checks for non-empty status fields and distinct pre/post message IDs.
- Add RED fail-closed conformance tests first for `execute("S-09")` in all three drivers.
- Preserve regression behavior for existing live-bound scenarios.

Out of scope:
- Protocol or wire-format changes.
- New dependencies.
- Scenario activation beyond `S-09`.

## Acceptance Criteria
- AC-1: Given `KAMN_E2E_SDK_DIRECT_LIVE` is truthy, `SdkDirectDriver::execute("S-09")` runs a dedicated transport-failover probe and returns `fail` when `S-09` validation errors.
- AC-2: Given `KAMN_E2E_CLI_SCRIPTED_LIVE` is truthy, `CliScriptedDriver::execute("S-09")` runs a dedicated transport-failover probe and returns `fail` when `S-09` validation errors.
- AC-3: Given `KAMN_E2E_MCP_AGENT_LIVE` is truthy, `McpAgentDriver::execute("S-09")` runs a dedicated transport-failover probe and returns `fail` when `S-09` validation errors.
- AC-4: Existing live-bound scenarios (`S-01`..`S-08`) and non-live scenario behavior remain regression-green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `SdkDirectDriver::execute("S-09")` with failover probe error | `status="fail"` |
| C-02 | AC-2 | Functional | `CliScriptedDriver::execute("S-09")` with failover probe error | `status="fail"` |
| C-03 | AC-3 | Functional | `McpAgentDriver::execute("S-09")` with failover probe error | `status="fail"` |
| C-04 | AC-4 | Regression | `cargo test -p kamn-e2e-harness -- --nocapture` | Existing scenario/toggle contracts remain green |
| C-05 | AC-4 | Conformance | `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture` | Spec-volume/docs guardrails remain green after lifecycle artifacts |

## Test Mapping
- `cargo test -p kamn-e2e-harness drivers::sdk_direct::tests::spec_c09_live_s09_driver_path_fails_closed_when_transport_failover_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness drivers::cli_scripted::tests::spec_c09_live_s09_driver_path_fails_closed_when_transport_failover_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness drivers::mcp_agent::tests::spec_c16_live_s09_driver_path_fails_closed_when_transport_failover_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness -- --nocapture`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics / Observable Signals
- `S-09` is live-bound in all three driver routing maps.
- Transport-failover probes enforce deterministic fail-closed validation of pre/post continuity and failover-boundary health checks.
- New `S-09` conformance tests and regression suites are green.
