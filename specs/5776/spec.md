# Spec: #5776 Add Query-Task and Query-Agent-Profile Portable-Agent Surfaces

- Issue: #5776
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`kamn-agent-lib` already supports `query_task` and `query_agent_profile`, but those operations are not exposed through the portable external interfaces (`kamn-cli` and `kamn-mcp-server`). This blocks end users and MCP clients from querying task lifecycle state and agent profile/reputation through first-class command/tool surfaces.

## Scope
### In Scope
- Add CLI command kinds and command modules for:
  - `query-task <task_id>`
  - `query-agent-profile <did>`
- Add MCP tool inventory and dispatch support for:
  - `query_task`
  - `query_agent_profile`
- Extend MCP protocol/inventory/dispatch tests for new tools and invalid-request paths.
- Extend CLI subcommand/activation contract tests for new commands and missing-arg behavior.
- Add at least one real-backend MCP integration contract assertion for a new query route.
- Add lifecycle artifacts and milestone tracking updates.
- Preserve top-level `specs/` cap (`<= 693`) using compensating archive cleanup after adding `specs/5776`.

### Out of Scope
- New service API routes (existing SDK/client routes are reused).
- CI/workflow changes.

## Acceptance Criteria
### AC-1 CLI query surfaces exposed
Given portable CLI users,
When commands are parsed and dispatched,
Then `query-task` and `query-agent-profile` execute and return deterministic key-value/json projections.

### AC-2 MCP query tools exposed
Given MCP clients,
When `tools/list` and `tools/call` are used,
Then `query_task` and `query_agent_profile` are available and dispatch through backend adapters.

### AC-3 Fail-closed invalid-request behavior
Given missing required arguments/fields,
When query commands/tools are invoked,
Then deterministic invalid-input/invalid-request responses are returned.

### AC-4 Real-backend integration coverage
Given the real backend integration fixture,
When at least one new query tool is dispatched,
Then response contract verifies real route composition and projection.

### AC-5 Specs cap preserved
Given one new lifecycle spec directory is added,
When implementation completes,
Then top-level `specs/` directory count remains `<= 693` via compensating archive cleanup.

## Conformance Cases
- C-01 (AC-1): `cargo test -p kamn-cli --test subcommand_surface_contract`.
- C-02 (AC-1/AC-3): `cargo test -p kamn-cli --test command_activation_contract`.
- C-03 (AC-2): `cargo test -p kamn-mcp-server --test tool_inventory_contract`.
- C-04 (AC-2/AC-3): `cargo test -p kamn-mcp-server --test tool_dispatch_contract`.
- C-05 (AC-2/AC-3): `cargo test -p kamn-mcp-server --test stdio_protocol_contract`.
- C-06 (AC-4): `cargo test -p kamn-mcp-server --test real_backend_integration_contract`.
- C-07 (AC-5): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>`.
- C-08 (AC-1..AC-5): `cargo fmt --all --check`.
- C-09 (AC-1..AC-5): `cargo clippy -p kamn-cli -p kamn-mcp-server --tests -- -D warnings`.
- C-10 (AC-1..AC-5): `cargo test --workspace --locked --all-features --no-fail-fast`.

## Success Metrics / Observable Signals
- New CLI commands parse/dispatch successfully.
- MCP tool inventory grows beyond existing 12-tool baseline with deterministic contracts.
- Workspace gate remains green after integration.
