# Spec: Issue #6004 - Replace MCP protocol JSON string parsing with serde_json

- Issue: #6004
- Status: Reviewed
- Type: story
- Priority: P0
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: #5917

## Problem Statement
`crates/kamn-mcp-server/src/protocol.rs` currently parses JSON-RPC request fields via string/token extraction helpers. This is brittle around escaped strings, nested params, and malformed input boundaries.

## Scope
In scope:
- Switch protocol request parsing to typed `serde_json` values/structs.
- Preserve MCP JSON-RPC behavior for `initialize`, `tools/list`, and `tools/call`.
- Preserve existing framed response/output contracts.
- Add regression tests covering escaped strings, nested params, and malformed JSON handling.

Out of scope:
- Tool backend semantics.
- stdio transport framing changes.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Request decode path uses `serde_json` parsing instead of string-slicing field extraction.
- AC-2: `tools/call` parameter extraction correctly handles escaped/nested string values without truncation or key confusion.
- AC-3: Malformed JSON input returns deterministic JSON-RPC parse/invalid-request errors.
- AC-4: Existing protocol framed-mode contract behavior remains passing.

## Conformance Cases
- C-01 (Unit, AC-1): typed parse helper accepts valid JSON-RPC requests and returns method/id/params structure.
- C-02 (Functional, AC-2): `tools/call` with escaped payload and nested params maps expected dispatch payload keys/values.
- C-03 (Regression, AC-3): malformed JSON returns framed JSON-RPC parse error (`-32700`) without panics.
- C-04 (Integration, AC-4): framed request processing keeps `initialize`, `tools/list`, `tools/call` response framing behavior unchanged.

## Success Metrics / Observable Signals
- No protocol-path dependence on manual `json_*_field` string scanning.
- Escaped/nested request payloads are interpreted correctly.
- Protocol tests remain green with new regression coverage.
