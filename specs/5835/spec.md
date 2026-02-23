# Spec: Issue #5835 - Activate Opt-In Live S-14 Batch-Merkle Scenario Across E2E Drivers

- Issue: #5835
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
Live driver activation currently covers scenarios through `S-13`. `S-14` (`Batch Merkle Anchoring`) remains scaffold-only, so batched submission, deterministic batch-root anchoring, and proof-verification continuity are not exercised in live-bound conformance execution.

## Scope
In scope:
- Add live `S-14` routing/probe support in:
  - `sdk-direct`
  - `cli-scripted`
  - `mcp-agent`
- Implement deterministic `S-14` probe flow that validates:
  - two batched message submissions succeed,
  - both message lifecycle queries remain coherent,
  - both proof verifications succeed with a shared deterministic batch-root marker.
- Reuse existing operation surfaces where sufficient (`send_message`, `query_message`, `verify_proof`); add no new wire/protocol surfaces unless required.
- Add RED fail-closed conformance tests first for `execute("S-14")` in all three drivers.

Out of scope:
- New dependencies.
- Protocol framing changes.
- Scenario activation beyond `S-14`.

## Acceptance Criteria
- AC-1: Given `KAMN_E2E_SDK_DIRECT_LIVE` is truthy, `SdkDirectDriver::execute("S-14")` runs a dedicated batch-merkle probe and returns `fail` when `S-14` validation errors.
- AC-2: Given `KAMN_E2E_CLI_SCRIPTED_LIVE` is truthy, `CliScriptedDriver::execute("S-14")` runs a dedicated batch-merkle probe and returns `fail` when `S-14` validation errors.
- AC-3: Given `KAMN_E2E_MCP_AGENT_LIVE` is truthy, `McpAgentDriver::execute("S-14")` runs a dedicated batch-merkle probe and returns `fail` when `S-14` validation errors.
- AC-4: Existing CLI/MCP/API operation contracts used by S-14 probes (`send/query/verify`) remain coherent and deterministic.
- AC-5: Existing live-bound scenarios (`S-01`..`S-13`) and non-live scenario behavior remain regression-green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `SdkDirectDriver::execute("S-14")` with S-14 probe error | `status="fail"` |
| C-02 | AC-2 | Functional | `CliScriptedDriver::execute("S-14")` with S-14 probe error | `status="fail"` |
| C-03 | AC-3 | Functional | `McpAgentDriver::execute("S-14")` with S-14 probe error | `status="fail"` |
| C-04 | AC-4 | Integration | S-14 probe uses existing send/query/verify operation contracts | deterministic pass + coherent response markers |
| C-05 | AC-5 | Regression | `cargo test -p kamn-e2e-harness -- --nocapture` | existing scenario/toggle contracts remain green |
| C-06 | AC-5 | Conformance | `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture` | review/docs guardrails remain green |

## Test Mapping
- `cargo test -p kamn-e2e-harness live_s14_driver_path_fails_closed_when_batch_merkle_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness -- --nocapture`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics / Observable Signals
- `S-14` is live-bound in all three driver routing maps.
- `S-14` probes verify deterministic batched-send/query continuity plus shared batch-root proof-verification continuity.
- No regression in existing scenario contracts or review/docs contract lanes.
