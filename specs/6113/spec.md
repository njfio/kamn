# Spec: Issue #6113 - MCP per-tool input schemas

Status: Reviewed
Issue: #6113
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
`crates/kamn-mcp-server/src/tools.rs` currently publishes the same generic input schema (`{"type":"object","additionalProperties":true}`) for all 21 MCP tools. This prevents clients from discovering required parameters and weakens contract safety.

## Scope
In scope:
- Replace generic MCP input schemas with per-tool deterministic JSON schemas.
- Keep tool names/descriptions stable.
- Add conformance/regression tests proving schema specificity and argument parity.

Out of scope:
- Changing MCP tool names.
- Changing backend dispatch semantics.
- Redesigning output schema envelope format.

## Acceptance Criteria
- AC-1: Every MCP tool descriptor exposes a tool-specific input schema that identifies required arguments for that tool.
- AC-2: `tools/list` returns deterministic per-tool input schema payloads (not the same generic schema for all tools).
- AC-3: Contract/regression tests fail if schemas regress back to fully generic descriptors.

## Conformance Cases
- C-01 (AC-1): `register` and `health` have no required fields and reject unknown fields via `additionalProperties:false`.
- C-02 (AC-1): Single-argument tools expose the expected required field (`payload`, `channel_id`, `message_id`, `task_id`, `did`, `content_id`, `bridge_id`, `escrow_id`).
- C-03 (AC-1): `verify_proof` requires `message_id`, `tx_hash`, `block_height`, and `finality`, with `block_height` typed as integer.
- C-04 (AC-2): `tools/list` response contains per-tool schemas for known tools (example: `query_task` includes `task_id`).
- C-05 (AC-3): No tool in the registry uses the legacy generic input schema value.

## Success Metrics
- `cargo test -p kamn-mcp-server --test tool_inventory_contract`
- `cargo test -p kamn-mcp-server --test stdio_protocol_contract spec_c02_mcp_tools_list_framed_tool_inventory_contract -- --exact`
