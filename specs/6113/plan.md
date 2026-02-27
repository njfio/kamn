# Plan: Issue #6113

## Approach
1. Replace the static generic input schema assignment in `tools.rs` with a deterministic tool-to-schema mapping.
2. Keep output schema deterministic and stable.
3. Add/adjust contract tests in `tool_inventory_contract.rs` to assert required-field parity and guard against generic schema regression.
4. Validate via focused MCP tests.

## Affected Modules
- `crates/kamn-mcp-server/src/tools.rs`
- `crates/kamn-mcp-server/tests/tool_inventory_contract.rs`
- (optional) `crates/kamn-mcp-server/tests/stdio_protocol_contract.rs` assertions for tools/list payload visibility

## Risks
- Risk: schema field mismatch with dispatcher-required arguments.
  - Mitigation: table-driven test aligns expected required fields per tool with current dispatch contract.
- Risk: accidental non-deterministic schema serialization.
  - Mitigation: use fixed static JSON literals.

## Interfaces/Contracts
- Preserved: `ToolDescriptor` and `build_tool_registry()` public surface.
- Changed behavior: `input_schema` content becomes tool-specific.
