# Spec: Issue 6199 - MCP Framed Input Must Enforce Max Content-Length

- Issue: #6199
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P1
- Area: security

## Problem Statement

`kamn-mcp-server` allocates `vec![0_u8; content_length]` from untrusted framed headers
without a size cap, enabling unbounded allocation attempts.

## Scope

In scope:
1. Add deterministic max framed payload length enforcement.
2. Reject oversize content-length headers before allocation.
3. Add unit regression tests for cap validation helpers.

Out of scope:
1. Streaming parser redesign.
2. Compressed frame support.

## Acceptance Criteria

### AC-1 Oversize Header Rejected
Given framed input with `content-length` greater than cap,
When request is parsed,
Then startup loop fails closed before allocation.

### AC-2 In-Bounds Header Accepted
Given framed input with valid `content-length`,
When request is parsed,
Then payload allocation proceeds and request processing remains unchanged.

### AC-3 Cap Validation Is Unit-Tested
Given the cap helper,
When called with boundary values (`cap`, `cap+1`),
Then helper returns success/failure deterministically.

## Conformance Cases

- C-01 (AC-1, Unit): `main::tests::regression_issue_6199_validate_content_length_rejects_oversize_values`
- C-02 (AC-2, Integration): `main_stdio_persistent_contract::spec_c10_main_stdio_session_processes_multiple_framed_requests_without_eof`
- C-03 (AC-3, Unit): `main::tests::regression_issue_6199_validate_content_length_accepts_boundary_values`
