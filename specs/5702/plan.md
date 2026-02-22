# Plan: #5702 Upgrade MCP-Agent Live S-01 Probe to Framed JSON-RPC Flow

## Approach
1. Add RED helper tests in `mcp_agent.rs` for framed request construction and
   framed response extraction/validation.
2. Refactor MCP live probe to:
   - write framed `initialize` and `tools/call` requests to stdin
   - parse framed responses from stdout
   - assert required success markers
3. Keep existing toggle and scenario routing behavior unchanged.
4. Run full harness regression suite plus formatting/lint/mutation checks.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `crates/kamn-e2e-harness/tests/mcp_agent_live_toggle_contract.rs` (regression only)

## Risks and Mitigations
- Risk: brittle string matching in protocol validation.
  Mitigation: validate minimal stable markers (`jsonrpc`, method IDs, `ok=true`)
  and include parse-error tests.

- Risk: framing parser mistakes causing false positives.
  Mitigation: enforce `Content-Length` accounting in helper parser.

## Interfaces / Contracts
- `McpAgentDriver` API remains unchanged.
- Probe internals switch from line-mode payload to framed JSON-RPC payloads.

## ADR
- Not required: no dependency or cross-crate protocol schema addition.
