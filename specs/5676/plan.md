# Plan: #5676 Activate MCP + CLI Task/Escrow Operations

## Approach
1. Add RED MCP tests for success + invalid-request behavior on all four operations.
2. Implement MCP backend trait + dispatch routing for task/escrow tools using `KamnAgentHandle` methods.
3. Add RED CLI command activation tests for `accept-task`, `complete-task`, `fund-escrow`, `release-escrow`.
4. Implement CLI command modules and command router wiring.
5. Run targeted MCP/CLI tests and lint/format checks.

## Affected Modules
- `crates/kamn-mcp-server/src/dispatch.rs`
- `crates/kamn-mcp-server/tests/tool_dispatch_contract.rs`
- `crates/kamn-cli/src/commands/{accept_task,complete_task,fund_escrow,release_escrow}.rs` (new)
- `crates/kamn-cli/src/commands/mod.rs`
- `crates/kamn-cli/tests/command_activation_contract.rs`
- `docs/prd/e2e-live-testing-prd.md`

## Risks and Mitigations
- Risk: argument schema drift between CLI and MCP for the same operations.
- Mitigation: explicit malformed-input contract tests in both crates with deterministic envelope/error assertions.
- Risk: regression to existing tool/command handling.
- Mitigation: preserve and re-run current activation contract suites unchanged.

## Interfaces / Contracts
- MCP tool responses:
  - `accept_task`: `{ "task_id": "...", "state": "accepted" }`
  - `complete_task`: `{ "task_id": "...", "state": "completed" }`
  - `fund_escrow`: `{ "escrow_id": "...", "state": "funded" }`
  - `release_escrow`: `{ "escrow_id": "...", "state": "released" }`
- CLI command outputs mirror same JSON response shapes.

## ADR
- Not required; additive activation on existing command/dispatch architecture.
