# Spec: #5702 Upgrade MCP-Agent Live S-01 Probe to Framed JSON-RPC Flow

- Issue: #5702
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
The MCP-agent live S-01 probe currently exercises the legacy line-mode tool
request path. The harness should validate the framed MCP JSON-RPC protocol path
(`initialize` + `tools/call`) to align with the intended MCP integration layer.

## Scope
### In Scope
- Upgrade `run_live_s01_mcp_probe` to send framed JSON-RPC requests:
  - `initialize`
  - `tools/call` (`health`)
- Validate framed responses for protocol and health-success markers.
- Add helper tests for framed request/response parsing behavior.
- Preserve existing driver toggle behavior and public interfaces.

### Out of Scope
- New tools or protocol methods.
- Changes to run output schema.
- Non-MCP driver behavior changes.

## Acceptance Criteria
### AC-1 Initialize handshake
Given MCP live probe execution,
When `initialize` framed JSON-RPC request is sent,
Then response includes JSON-RPC success markers with `serverInfo`.

### AC-2 Health tools/call
Given MCP live probe execution,
When framed `tools/call` health request is sent,
Then response includes JSON-RPC success with nested `ok=true` tool result.

### AC-3 Framed output validation
Given malformed or missing framed output,
When probe parser runs,
Then probe returns deterministic failure.

### AC-4 Regression stability
Given existing MCP live toggle conformance tests,
When framed probe path is introduced,
Then tests and full harness suite remain green.

## Conformance Cases
- C-01 (AC-1): framed initialize request generation/validation test.
- C-02 (AC-2): framed tools/call health validation test.
- C-03 (AC-3): malformed framed output parse returns error.
- C-04 (AC-4): `cargo test -p kamn-e2e-harness` passes.

## Success Metrics / Observable Signals
- MCP live probe no longer uses legacy line-mode payloads.
- Framed JSON-RPC markers are required for probe success.
- Harness regression suites remain green.
