# Spec: Issue 6214 - Parse MCP Success Responses as Structured JSON

- Issue: #6214
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P2
- Area: backend

## Problem Statement

`kamn-e2e-harness` MCP driver validated success by substring matching
`"ok":true` in payload text. This can misclassify responses when nested fields
contain `ok=true` while the canonical success field is false.

## Scope

In scope:
1. Replace substring success checks with structured JSON bool extraction.
2. Ensure JSON field extraction supports `result` scope in JSON-RPC payloads.
3. Add regressions proving nested `ok=true` does not bypass root `ok=false`.

Out of scope:
1. Full response schema validation for all MCP payload fields.
2. MCP protocol redesign.

## Acceptance Criteria

### AC-1 Tool Success Check Uses JSON Bool Field
Given MCP tool response payloads,
When `run_live_s04_mcp_tool_call` evaluates success,
Then it reads `ok` as structured JSON boolean and fails when root `ok` is false.

### AC-2 Probe Health Success Check Uses JSON Bool Field
Given MCP health payloads,
When `validate_probe_health_response` evaluates success,
Then it reads `ok` as structured JSON boolean and fails when root `ok` is false.

### AC-3 Nested `ok=true` Cannot Bypass Root Failure
Given payloads containing nested `ok=true` but root `ok=false`,
When success checks execute,
Then both tool-call and health validations fail closed.

## Conformance Cases

- C-01 (AC-1, Unit): `tests::unit_run_live_s04_mcp_tool_call_accepts_ok_true_payload`
- C-02 (AC-2, Unit): `tests::spec_c24_validate_probe_health_response_accepts_success_payload`
- C-03 (AC-3, Unit): `tests::regression_issue_6214_run_live_s04_mcp_tool_call_rejects_nested_ok_true_when_root_false`
- C-04 (AC-3, Unit): `tests::regression_issue_6214_validate_probe_health_response_rejects_nested_ok_true_when_root_false`

