# Spec: Issue #6135 - Replace MCP success string-match with field-aware JSON bool parsing

- Issue: #6135
- Status: Reviewed
- Type: task
- Priority: P2
- Area: backend
- Milestone: `specs/milestones/r68-r59-swarm-remediation-and-full-gap-closure/index.md`
- Last Updated: 2026-02-27
- Parent: #6100

## Problem Statement
`crates/kamn-e2e-harness/src/drivers/mcp_agent.rs` determines MCP tool-call success with `payload.contains("\"ok\":true")`. This is fragile and can be satisfied by irrelevant nested strings or formatting artifacts. The probe must verify the JSON `ok` field value directly.

## Scope
In scope:
- Replace string-match success checks in live MCP probe paths with field-aware JSON bool extraction for `ok`.
- Add conformance/regression tests proving the parser accepts real `ok:true` payloads and rejects malformed/quoted/non-boolean forms.
- Keep framed transport behavior and existing request/response wiring unchanged.

Out of scope:
- Full serde migration for all e2e harness JSON helpers.
- MCP protocol/schema redesign.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: MCP probe success checks validate `ok` as a boolean JSON field, not substring containment.
- AC-2: Probe validation fails when `ok` is absent, quoted (`"true"`), or `false`.
- AC-3: Existing successful tool-call live probe path remains green for valid `{"ok":true}` payloads.

## Conformance Cases
- C-01 (Unit, AC-1): `json_optional_bool_field` extracts top-level boolean field values for `true` and `false`.
- C-02 (Unit/Regression, AC-2): validation rejects payloads with missing `ok`, quoted `ok`, or `ok:false`.
- C-03 (Functional/Conformance, AC-3): `run_live_s04_mcp_tool_call` accepts framed payload with `{"ok":true}` and rejects framed payload with `{"ok":"true"}`.

## Success Metrics / Observable Signals
- No production success decision in MCP live probes uses `contains("\"ok\":true")`.
- New conformance tests for S-12 pass in `kamn-e2e-harness` test runs.
