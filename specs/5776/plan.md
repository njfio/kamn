# Plan: #5776 Add Query-Task and Query-Agent-Profile Portable-Agent Surfaces

## Approach
1. Add lifecycle artifacts and mark milestone delivery slice in progress.
2. RED: run affected CLI/MCP contract lanes to establish current absence/failure expectations.
3. Implement CLI surface additions (command enum parse/dispatch + two command modules + tests).
4. Implement MCP surface additions (tool inventory + backend trait/dispatch + tests).
5. Extend real-backend MCP integration fixture and contract assertion for one new query route.
6. Perform compensating archive cleanup and update `specs/archive/index.md`.
7. GREEN: rerun all affected lanes and verify workspace gate.
8. Closure: mark spec implemented, tasks complete, milestone slice complete, and close issue.

## Affected Modules / Files
- `crates/kamn-cli/src/lib.rs`
- `crates/kamn-cli/src/commands/mod.rs`
- `crates/kamn-cli/src/commands/query_task.rs` (new)
- `crates/kamn-cli/src/commands/query_agent_profile.rs` (new)
- `crates/kamn-cli/tests/subcommand_surface_contract.rs`
- `crates/kamn-cli/tests/command_activation_contract.rs`
- `crates/kamn-mcp-server/src/tools.rs`
- `crates/kamn-mcp-server/src/dispatch.rs`
- `crates/kamn-mcp-server/tests/tool_inventory_contract.rs`
- `crates/kamn-mcp-server/tests/tool_dispatch_contract.rs`
- `crates/kamn-mcp-server/tests/stdio_protocol_contract.rs`
- `crates/kamn-mcp-server/tests/real_backend_integration_contract.rs`
- `specs/archive/index.md`
- `specs/5776/spec.md`
- `specs/5776/plan.md`
- `specs/5776/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: command/tool naming inconsistency across CLI/MCP surfaces.
  - Mitigation: enforce deterministic explicit names in tests.
- Risk: fixture server route mismatch for real-backend integration.
  - Mitigation: add explicit route handlers and route-path assertions.
- Risk: specs cap regression due new lifecycle dir.
  - Mitigation: compensating archive cleanup + policy checker.

## Interfaces / Contracts
- CLI: new commands `query-task`, `query-agent-profile`.
- MCP: new tools `query_task`, `query_agent_profile`.

## ADR
No ADR required (no new dependency, architecture, protocol-wire change).
