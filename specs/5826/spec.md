# Spec: Issue #5826 - Activate Opt-In Live S-10 Topology-Coherence Scenario Across E2E Drivers

- Issue: #5826
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
The harness now supports live probes through `S-09`, but `S-10` (Multi-Node Topology Coherence) is still scaffold-only in live driver routing. This leaves cross-node propagation and topology-coherence checks outside live-bound conformance execution.

## Scope
In scope:
- Add live `S-10` routing/probe support in:
  - `sdk-direct`
  - `cli-scripted`
  - `mcp-agent`
- Implement deterministic `S-10` topology-coherence probes that execute:
  - primary-node send validation,
  - secondary/tertiary-node query validation for the same message id,
  - secondary/tertiary boundary health validation,
  - fail-closed status/message-id checks.
- Add RED fail-closed conformance tests first for `execute("S-10")` in all three drivers.
- Update live-toggle contracts so non-live-bound scenario checks move from `S-10` to `S-11`.

Out of scope:
- Protocol or wire-format changes.
- New dependencies.
- Scenario activation beyond `S-10`.

## Acceptance Criteria
- AC-1: Given `KAMN_E2E_SDK_DIRECT_LIVE` is truthy, `SdkDirectDriver::execute("S-10")` runs a dedicated topology-coherence probe and returns `fail` when `S-10` validation errors.
- AC-2: Given `KAMN_E2E_CLI_SCRIPTED_LIVE` is truthy, `CliScriptedDriver::execute("S-10")` runs a dedicated topology-coherence probe and returns `fail` when `S-10` validation errors.
- AC-3: Given `KAMN_E2E_MCP_AGENT_LIVE` is truthy, `McpAgentDriver::execute("S-10")` runs a dedicated topology-coherence probe and returns `fail` when `S-10` validation errors.
- AC-4: Existing live-bound scenarios (`S-01`..`S-09`) and non-live scenario behavior remain regression-green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `SdkDirectDriver::execute("S-10")` with topology probe error | `status="fail"` |
| C-02 | AC-2 | Functional | `CliScriptedDriver::execute("S-10")` with topology probe error | `status="fail"` |
| C-03 | AC-3 | Functional | `McpAgentDriver::execute("S-10")` with topology probe error | `status="fail"` |
| C-04 | AC-4 | Regression | `cargo test -p kamn-e2e-harness -- --nocapture` | Existing scenario/toggle contracts remain green |
| C-05 | AC-4 | Conformance | `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture` | Spec-volume/docs guardrails remain green after lifecycle artifacts |

## Test Mapping
- `cargo test -p kamn-e2e-harness drivers::sdk_direct::tests::spec_c10_live_s10_driver_path_fails_closed_when_topology_coherence_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness drivers::cli_scripted::tests::spec_c10_live_s10_driver_path_fails_closed_when_topology_coherence_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness drivers::mcp_agent::tests::spec_c17_live_s10_driver_path_fails_closed_when_topology_coherence_probe_errors -- --nocapture`
- `cargo test -p kamn-e2e-harness -- --nocapture`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics / Observable Signals
- `S-10` is live-bound in all three driver routing maps.
- Topology-coherence probes enforce deterministic fail-closed validation for cross-node query/health continuity.
- New `S-10` conformance tests and regression suites are green.
