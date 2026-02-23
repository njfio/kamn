# Spec: Issue #5828 - Activate Opt-In Live S-11 Signer-Rotation Scenario Across E2E Drivers

- Issue: #5828
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
Live driver activation currently covers scenarios through `S-10`. `S-11` (`Signer Key Rotation`) remains scaffold-only, so signer-rotation continuity and stale-signer rejection are not exercised in live-bound conformance execution.

## Scope
In scope:
- Add live `S-11` routing/probe support in:
  - `sdk-direct`
  - `cli-scripted`
  - `mcp-agent`
- Implement deterministic `S-11` probes that enforce:
  - baseline signer request succeeds,
  - rotated signer request succeeds,
  - stale/old signer replay request fails closed with explicit rejection marker.
- Add RED fail-closed conformance tests first for `execute("S-11")` in all three drivers.
- Update live-toggle contracts so non-live-bound scenario checks move from `S-11` to `S-12`.

Out of scope:
- Protocol or wire-format changes.
- New dependencies.
- Scenario activation beyond `S-11`.

## Acceptance Criteria
- AC-1: Given `KAMN_E2E_SDK_DIRECT_LIVE` is truthy, `SdkDirectDriver::execute("S-11")` runs a dedicated signer-rotation probe and returns `fail` when `S-11` validation errors.
- AC-2: Given `KAMN_E2E_CLI_SCRIPTED_LIVE` is truthy, `CliScriptedDriver::execute("S-11")` runs a dedicated signer-rotation probe and returns `fail` when `S-11` validation errors.
- AC-3: Given `KAMN_E2E_MCP_AGENT_LIVE` is truthy, `McpAgentDriver::execute("S-11")` runs a dedicated signer-rotation probe and returns `fail` when `S-11` validation errors.
- AC-4: Existing live-bound scenarios (`S-01`..`S-10`) and non-live scenario behavior remain regression-green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `SdkDirectDriver::execute("S-11")` with signer-rotation probe error | `status="fail"` |
| C-02 | AC-2 | Functional | `CliScriptedDriver::execute("S-11")` with signer-rotation probe error | `status="fail"` |
| C-03 | AC-3 | Functional | `McpAgentDriver::execute("S-11")` with signer-rotation probe error | `status="fail"` |
| C-04 | AC-4 | Regression | `cargo test -p kamn-e2e-harness -- --nocapture` | Existing scenario/toggle contracts remain green |
| C-05 | AC-4 | Conformance | `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture` | Spec-volume/docs guardrails remain green after lifecycle artifacts |

## Test Mapping
- `cargo test -p kamn-e2e-harness drivers::sdk_direct::tests::spec_c11_live_s11_driver_path_fails_closed_when_signer_rotation_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness drivers::cli_scripted::tests::spec_c11_live_s11_driver_path_fails_closed_when_signer_rotation_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness drivers::mcp_agent::tests::spec_c18_live_s11_driver_path_fails_closed_when_signer_rotation_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness -- --nocapture`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics / Observable Signals
- `S-11` is live-bound in all three driver routing maps.
- Signer-rotation probes enforce deterministic continuity (rotated signer accepted) and fail-closed stale signer rejection with explicit rejection marker.
- New `S-11` conformance tests and regression suites are green.
