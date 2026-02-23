# Spec: Issue #5837 - Activate Opt-In Live S-15 Performance-Smoke Scenario Across E2E Drivers

- Issue: #5837
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
Live driver activation now covers `S-01` through `S-14`. `S-15` (`Performance Smoke`) remains scaffold-only, so bounded-latency behavior is not exercised in live-bound conformance.

## Scope
In scope:
- Add live `S-15` routing/probe support in:
  - `sdk-direct`
  - `cli-scripted`
  - `mcp-agent`
- Implement deterministic S-15 probe flow that:
  - executes bounded message send/query continuity loops,
  - records per-iteration latency samples,
  - validates total/p50/p99 thresholds via env-configurable budgets.
- Add RED fail-closed conformance tests first for `execute("S-15")` in all three drivers.
- Preserve existing live/non-live behavior for `S-01..S-14`.

Out of scope:
- New dependencies.
- Protocol/wire-format changes.
- CI workflow changes.

## Acceptance Criteria
- AC-1: Given `KAMN_E2E_SDK_DIRECT_LIVE` is truthy, `SdkDirectDriver::execute("S-15")` runs dedicated performance-smoke probe logic and returns `fail` on probe validation errors.
- AC-2: Given `KAMN_E2E_CLI_SCRIPTED_LIVE` is truthy, `CliScriptedDriver::execute("S-15")` runs dedicated performance-smoke probe logic and returns `fail` on probe validation errors.
- AC-3: Given `KAMN_E2E_MCP_AGENT_LIVE` is truthy, `McpAgentDriver::execute("S-15")` runs dedicated performance-smoke probe logic and returns `fail` on probe validation errors.
- AC-4: S-15 probes validate deterministic latency-budget contracts (`max_total_ms`, `max_p50_ms`, `max_p99_ms`) with clear fail-closed errors.
- AC-5: Existing harness and docs/spec non-regression lanes remain green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `SdkDirectDriver::execute("S-15")` with S-15 probe error | `status="fail"` |
| C-02 | AC-2 | Functional | `CliScriptedDriver::execute("S-15")` with S-15 probe error | `status="fail"` |
| C-03 | AC-3 | Functional | `McpAgentDriver::execute("S-15")` with S-15 probe error | `status="fail"` |
| C-04 | AC-4 | Unit | latency samples exceed p50/p99/total thresholds | fail-closed contract error |
| C-05 | AC-4 | Integration | valid bounded-latency S-15 probe flow | pass |
| C-06 | AC-5 | Regression | `cargo test -p kamn-e2e-harness -- --nocapture` | existing scenario/toggle contracts green |
| C-07 | AC-5 | Conformance | `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture` | review/docs contract lane green |

## Test Mapping
- `cargo test -p kamn-e2e-harness live_s15_driver_path_fails_closed -- --nocapture`
- `cargo test -p kamn-e2e-harness s15 -- --nocapture`
- `cargo test -p kamn-e2e-harness -- --nocapture`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics / Observable Signals
- `S-15` routes live in all three drivers.
- S-15 probes enforce deterministic performance-smoke thresholds with fail-closed semantics.
- No regression in existing scenarios and docs-contract lanes.
