# Spec: Issue #5820 - Activate Opt-In Live S-07 Replay-Protection Scenario Across E2E Drivers

- Issue: #5820
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
The harness currently supports live probes for `S-01` through `S-06`, but `S-07` replay-protection remains scaffold-only. This leaves replay-guard semantics outside live-bound driver contracts.

## Scope
In scope:
- Add live `S-07` routing/probe support in:
  - `sdk-direct`
  - `cli-scripted`
  - `mcp-agent`
- Implement replay-protection probes that validate deterministic replay rejection marker `service_api_auth_replay_nonce_detected`.
- Add RED fail-closed conformance tests first for `execute("S-07")` in all three drivers.
- Preserve regression behavior for existing live-bound scenarios.

Out of scope:
- Protocol or wire-format changes.
- New dependencies.
- Scenario activation beyond `S-07`.

## Acceptance Criteria
- AC-1: Given `KAMN_E2E_SDK_DIRECT_LIVE` is truthy, `SdkDirectDriver::execute("S-07")` runs a dedicated replay probe and returns `fail` when replay validation errors.
- AC-2: Given `KAMN_E2E_CLI_SCRIPTED_LIVE` is truthy, `CliScriptedDriver::execute("S-07")` runs a dedicated replay probe and returns `fail` when replay validation errors.
- AC-3: Given `KAMN_E2E_MCP_AGENT_LIVE` is truthy, `McpAgentDriver::execute("S-07")` runs a dedicated replay probe and returns `fail` when replay validation errors.
- AC-4: Existing live-bound scenarios (`S-01`..`S-06`) and non-live scenario behavior remain regression-green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `SdkDirectDriver::execute("S-07")` with replay probe error | `status="fail"` |
| C-02 | AC-2 | Functional | `CliScriptedDriver::execute("S-07")` with replay probe error | `status="fail"` |
| C-03 | AC-3 | Functional | `McpAgentDriver::execute("S-07")` with replay probe error | `status="fail"` |
| C-04 | AC-4 | Regression | `cargo test -p kamn-e2e-harness -- --nocapture` | Existing scenario/toggle contracts remain green |
| C-05 | AC-4 | Conformance | `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture` | Spec-volume/docs guardrails remain green after lifecycle artifacts |

## Test Mapping
- `cargo test -p kamn-e2e-harness drivers::sdk_direct::tests::spec_c07_live_s07_driver_path_fails_closed_when_replay_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness drivers::cli_scripted::tests::spec_c07_live_s07_driver_path_fails_closed_when_replay_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness drivers::mcp_agent::tests::spec_c14_live_s07_driver_path_fails_closed_when_replay_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness -- --nocapture`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics / Observable Signals
- `S-07` is live-bound in all three driver routing maps.
- Replay probes validate deterministic replay reason-marker semantics.
- New `S-07` conformance tests and regression suites are green.
