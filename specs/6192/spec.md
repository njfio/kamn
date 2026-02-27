# Spec: Issue 6192 - MCP Tool Schemas Must Be Discoverable

- Issue: #6192
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P1
- Area: backend

## Problem Statement

All MCP tools currently advertise `{"type":"object","additionalProperties":true}` for
input/output schemas. Clients cannot discover required fields per tool.

## Scope

In scope:
1. Replace generic input schemas with concrete per-tool field schemas.
2. Replace fully-generic output schema with deterministic MCP envelope schema.
3. Update tests to validate discoverability of key field contracts.

Out of scope:
1. Runtime request validator changes.
2. Dynamic schema generation from Rust types.

## Acceptance Criteria

### AC-1 Per-Tool Input Schemas Are Concrete
Given any MCP tool descriptor,
When reading `input_schema`,
Then schema advertises required/optional fields relevant to that tool.

### AC-2 Output Schema Is Structured
Given any MCP tool descriptor,
When reading `output_schema`,
Then schema describes deterministic success/error envelope fields.

### AC-3 Schema Contract Tests Enforce Discoverability
Given tool inventory contract tests,
When tests run,
Then they fail if schemas regress to fully-generic descriptors.

## Conformance Cases

- C-01 (AC-1, Functional): `tool_inventory_contract::spec_c04_mcp_tool_registry_has_deterministic_schema_descriptors`
- C-02 (AC-2, Functional): `tool_inventory_contract::spec_c10_mcp_tool_registry_output_schema_declares_envelope_shape`
- C-03 (AC-3, Functional): `tool_inventory_contract::spec_c11_mcp_tool_registry_input_schemas_expose_required_fields_for_representative_tools`
