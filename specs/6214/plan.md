# Plan: Issue 6214 - Parse MCP Success Responses as Structured JSON

- Issue: #6214
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Extend MCP JSON helper surface with an optional boolean field extractor.
2. Ensure helper field lookup supports JSON-RPC `result` object scope.
3. Export bool helper from `kamn-mcp-server` for reuse by e2e harness.
4. Replace substring-based success checks in MCP driver with bool helper checks.
5. Add regressions for nested `ok=true` with root `ok=false`.

## Affected Modules

- `crates/kamn-mcp-server/src/json_helpers.rs`
- `crates/kamn-mcp-server/src/lib.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`

## Risks and Mitigations

1. Risk: bool field lookup order could change existing behavior.
   - Mitigation: keep deterministic lookup order with root and `result` first.
2. Risk: helper export broadens public API.
   - Mitigation: export one focused helper (`json_optional_bool_field`) only.

