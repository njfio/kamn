# Spec: #5700 Activate Opt-In Live MCP-Agent S-01 Driver Execution

- Issue: #5700
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`McpAgentDriver` in `kamn-e2e-harness` currently returns deterministic `"pass"`
for all scenarios. To continue phase-3 live activation, S-01 should support an
opt-in live probe path that exercises real MCP agent plumbing while preserving
deterministic behavior by default.

## Scope
### In Scope
- Add opt-in live toggle for MCP-agent driver (`mcp-tau`, `mcp-any`).
- Implement S-01 live probe using a real `kamn-mcp-server` invocation.
- Add injectable probe support for deterministic conformance tests.
- Preserve deterministic pass behavior by default.

### Out of Scope
- Live execution for non-S-01 scenarios.
- Run output JSON schema changes.
- SDK-direct/CLI-scripted driver behavior changes.

## Acceptance Criteria
### AC-1 Live toggle gating
Given MCP-agent driver execution,
When live toggle is disabled,
Then execution remains deterministic pass without invoking live probe runner.

### AC-2 S-01 live probe execution
Given live mode is enabled and scenario is `S-01`,
When live probe succeeds,
Then scenario returns pass.
When live probe fails (spawn/exit/backend error),
Then scenario returns fail.

### AC-3 Non-S-01 compatibility
Given live mode is enabled and scenario is not `S-01`,
When executed,
Then deterministic pass behavior is preserved.

### AC-4 Regression stability
Given existing harness contract suites,
When MCP-agent live toggle support is added,
Then existing harness suites remain green.

## Conformance Cases
- C-01 (AC-1): disabled-live MCP-agent execution does not invoke runner and
  returns pass.
- C-02 (AC-2): enabled-live S-01 with successful probe returns pass.
- C-03 (AC-2): enabled-live S-01 with failing probe returns fail.
- C-04 (AC-3): enabled-live non-S-01 does not invoke probe and returns pass.
- C-05 (AC-4): `cargo test -p kamn-e2e-harness` passes.

## Success Metrics / Observable Signals
- MCP-agent driver can emit real fail for S-01 in live mode.
- Deterministic default behavior remains unchanged.
- Harness crate tests continue to pass.
