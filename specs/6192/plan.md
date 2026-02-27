# Plan: Issue 6192 - MCP Tool Schemas Must Be Discoverable

- Issue: #6192
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Replace schema constants in `crates/kamn-mcp-server/src/tools.rs`:
   - define concrete input schemas grouped by argument shape (`payload`, `id`, `verify_proof`, no-args),
   - define one deterministic output envelope schema for all tools.
2. Update `tool_descriptor_for_name` to map each tool to specific input schema.
3. Rewrite schema contract tests in `tests/tool_inventory_contract.rs` to assert:
   - schema is not generic wildcard,
   - representative tools include expected required fields.

## Affected Modules

- `crates/kamn-mcp-server/src/tools.rs`
- `crates/kamn-mcp-server/tests/tool_inventory_contract.rs`

## Risks and Mitigations

1. Risk: schema drifts from dispatch arg names.
   - Mitigation: derive required field sets directly from `dispatch_tool_request_json` argument keys.
2. Risk: over-constraining future optional parameters.
   - Mitigation: keep `additionalProperties: false` for strict discoverability and update intentionally with explicit tests.
