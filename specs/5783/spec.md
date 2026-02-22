# Spec: Issue #5783 — Extend Opt-In Live S-06 Proof-Verification Execution Across E2E Drivers

- Issue: #5783
- Milestone: `r52-e2e-live-runtime-integration-hardening`
- Priority: P1
- Status: Implemented
- Last Updated: 2026-02-22

## Problem Statement
`kamn-e2e-harness` currently executes real live probes for `S-01` and `S-04`, but not for `S-06` (Kolme Proof Verification). In live mode, `S-06` still follows deterministic scaffold behavior, leaving proof-verification execution under-validated across SDK-direct, CLI-scripted, and MCP-agent modes.

## Scope
- Add opt-in live `S-06` execution to:
  - `sdk-direct` driver (`kamn-agent-lib` proof verification operation)
  - `cli-scripted` driver (`kamn-cli verify-proof` command probe)
  - `mcp-agent` driver (`kamn-mcp-server` `tools/call` verify_proof probe)
- Ensure live mode remains opt-in and fail-closed on `S-06` probe errors.
- Preserve behavior for existing live-bound scenarios (`S-01`, `S-04`) and non-live paths.

## Out of Scope
- Service API route/schema/protocol changes.
- Full live execution implementation for scenarios beyond `S-06`.
- Shell/workflow/template surface changes.

## Acceptance Criteria

### AC-1: SDK-direct driver executes live S-06 proof verification when enabled
Given `KAMN_E2E_SDK_DIRECT_LIVE` is truthy,
When `SdkDirectDriver::execute("S-06")` runs,
Then it must execute a dedicated live S-06 proof-verification probe and return `fail` on probe error.

### AC-2: CLI-scripted driver executes live S-06 proof verification when enabled
Given `KAMN_E2E_CLI_SCRIPTED_LIVE` is truthy,
When `CliScriptedDriver::execute("S-06")` runs,
Then it must execute a dedicated live S-06 proof-verification probe and return `fail` on probe error.

### AC-3: MCP-agent driver executes live S-06 proof verification when enabled
Given `KAMN_E2E_MCP_AGENT_LIVE` is truthy,
When `McpAgentDriver::execute("S-06")` runs,
Then it must execute a dedicated live S-06 proof-verification probe and return `fail` on probe error.

### AC-4: Existing live/non-live behavior remains stable
Given live mode disabled (or scenario IDs outside live-bound probes),
When drivers execute scenarios,
Then behavior remains backward-compatible and deterministic.

## Conformance Cases

| ID | AC | Tier | Case |
|---|---|---|---|
| C-01 | AC-1 | Functional | `sdk_direct` tests assert S-06 uses live probe and fails on error when enabled. |
| C-02 | AC-2 | Functional | `cli_scripted` tests assert S-06 uses live probe and fails on error when enabled. |
| C-03 | AC-3 | Functional | `mcp_agent` tests assert S-06 uses live probe and fails on error when enabled. |
| C-04 | AC-4 | Regression | Existing S-01/S-04 live behavior and disabled-live paths remain passing. |

## Success Metrics / Observable Signals
- `cargo test -p kamn-e2e-harness` passes with new S-06 assertions.
- `cargo fmt --all --check` and `cargo clippy -p kamn-e2e-harness --tests -- -D warnings` pass.
- Workspace gate remains green after integration.
