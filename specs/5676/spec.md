# Spec: #5676 Activate MCP + CLI Task/Escrow Operations

- Issue: #5676
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`kamn-agent-lib` now exposes `accept_task`, `complete_task`, `fund_escrow`, and `release_escrow`, but the MCP dispatcher and CLI command surface still return unsupported/pending behavior for these operations.

## Scope
### In Scope
- Implement MCP dispatch handlers for:
  - `accept_task`
  - `complete_task`
  - `fund_escrow`
  - `release_escrow`
- Add deterministic invalid-request handling for required arguments.
- Activate matching CLI commands and route argument parsing through `KamnAgentHandle`.
- Extend MCP and CLI contract tests for success + malformed inputs.
- Update PRD implementation progress markers.

### Out of Scope
- New service routes (already delivered in #5674).
- Non-deterministic workflow orchestration or autonomous task policies.

## Acceptance Criteria
### AC-1 MCP tool activation
Given valid MCP requests for task/escrow operations,
When dispatch executes,
Then each tool returns success envelopes backed by `KamnAgentHandle` operations.

### AC-2 MCP invalid-request determinism
Given malformed or missing arguments,
When dispatch parses task/escrow tool payloads,
Then it returns deterministic `invalid_request` error envelopes.

### AC-3 CLI command activation
Given valid CLI invocations for `accept-task`, `complete-task`, `fund-escrow`, and `release-escrow`,
When commands run,
Then they call `KamnAgentHandle` and print deterministic JSON outputs.

### AC-4 Regression preservation
Given existing MCP and CLI tool/command behavior,
When new task/escrow operations are activated,
Then previously implemented operations remain green.

## Conformance Cases
- C-01 (AC-1): MCP contract tests verify success dispatch for all four operations.
- C-02 (AC-2): MCP contract tests verify deterministic invalid-request envelopes for malformed/missing args.
- C-03 (AC-3): CLI contract tests verify all four commands invoke service routes and validate required args.
- C-04 (AC-4): Existing MCP + CLI activation tests remain passing.

## Success Metrics
- `cargo test -p kamn-mcp-server --test tool_dispatch_contract` passes.
- `cargo test -p kamn-cli --test command_activation_contract` passes.
- `cargo fmt --all --check` and crate-scoped clippy pass.
