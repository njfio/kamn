# Spec: Issue #5781 — Extend Opt-In Live S-04 Task Lifecycle Execution Across E2E Drivers

- Issue: #5781
- Milestone: `r52-e2e-live-runtime-integration-hardening`
- Priority: P1
- Status: Implemented
- Last Updated: 2026-02-22

## Problem Statement
`kamn-e2e-harness` currently executes real live probes only for scenario `S-01`. In live mode, `S-04` (Task Lifecycle) still passes through deterministic scaffold behavior. This leaves PRD-critical task/escrow paths under-validated for real runtime execution across the three driver modes.

## Scope
- Add opt-in live `S-04` execution to:
  - `sdk-direct` driver (`kamn-agent-lib` task + escrow operations)
  - `cli-scripted` driver (`kamn-cli` task + escrow commands)
  - `mcp-agent` driver (`kamn-mcp-server` task + escrow tool calls)
- Ensure live mode remains opt-in and fail-closed when probes fail.
- Keep non-live behavior and non-`S-04` scenario behavior unchanged.

## Out of Scope
- Service API route/schema changes.
- Full live implementation of all scenarios beyond `S-04`.
- Workflow/shell surface changes.

## Acceptance Criteria

### AC-1: SDK-direct driver executes live S-04 probe when enabled
Given `KAMN_E2E_SDK_DIRECT_LIVE` is truthy,
When `SdkDirectDriver::execute("S-04")` runs,
Then it must execute a dedicated live S-04 task-lifecycle probe and return `fail` on probe error.

### AC-2: CLI-scripted driver executes live S-04 probe when enabled
Given `KAMN_E2E_CLI_SCRIPTED_LIVE` is truthy,
When `CliScriptedDriver::execute("S-04")` runs,
Then it must execute a dedicated live S-04 task-lifecycle probe and return `fail` on probe error.

### AC-3: MCP-agent driver executes live S-04 probe when enabled
Given `KAMN_E2E_MCP_AGENT_LIVE` is truthy,
When `McpAgentDriver::execute("S-04")` runs,
Then it must execute a dedicated live S-04 task-lifecycle probe and return `fail` on probe error.

### AC-4: Existing S-01 live behavior and non-live behavior remain stable
Given live mode disabled (or scenario IDs other than live-bound probes),
When drivers execute scenarios,
Then behavior remains as before (`pass` scaffold path), preserving backward compatibility.

## Conformance Cases

| ID | AC | Tier | Case |
|---|---|---|---|
| C-01 | AC-1 | Functional | `sdk_direct` tests assert S-04 uses live probe and fails on error when enabled. |
| C-02 | AC-2 | Functional | `cli_scripted` tests assert S-04 uses live probe and fails on error when enabled. |
| C-03 | AC-3 | Functional | `mcp_agent` tests assert S-04 uses live probe and fails on error when enabled. |
| C-04 | AC-4 | Regression | Existing driver tests continue passing for S-01 and disabled-live paths. |

## Success Metrics / Observable Signals
- `cargo test -p kamn-e2e-harness` targeted driver tests pass with new S-04 assertions.
- `cargo fmt --all --check` and targeted clippy lane pass.
- Workspace gate remains green after integration.
