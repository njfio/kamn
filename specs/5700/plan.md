# Plan: #5700 Activate Opt-In Live MCP-Agent S-01 Driver Execution

## Approach
1. Refactor `McpAgentDriver` into configurable runtime driver with:
   - live toggle flag
   - injectable probe runner for tests
2. Implement default live probe by spawning `kamn-mcp-server` with
   endpoint/agent/key flags and sending a deterministic health tool request.
3. Keep deterministic pass behavior as default when live toggle is disabled.
4. Update run-contract driver creation to instantiate MCP driver via env-aware
   constructor.
5. Add conformance tests for toggle/runner behavior and run full harness tests.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `crates/kamn-e2e-harness/src/run_contract.rs`
- `crates/kamn-e2e-harness/tests/*` (new MCP live conformance tests)

## Risks and Mitigations
- Risk: probe behavior varies when binary/endpoint are unavailable.
  Mitigation: live mode is opt-in; failures map to `fail` only in live mode.

- Risk: run-contract regressions from constructor changes.
  Mitigation: full harness regression suite + existing command contracts.

## Interfaces / Contracts
- `HarnessDriver` trait unchanged.
- `McpAgentDriver` gains env-aware constructor and injectable probe path.
- Deterministic mode behavior remains stable by default.

## ADR
- Not required: no new dependency, no cross-crate protocol schema changes.
