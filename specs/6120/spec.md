# Spec: Issue #6120 - Bound MCP framed Content-Length

- Issue: #6120
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r68-r59-swarm-remediation-and-full-gap-closure/index.md`
- Last Updated: 2026-02-27
- Parent: #6100

## Problem Statement
`kamn-mcp-server` accepts framed stdio requests with `Content-Length` and allocates `vec![0_u8; content_length]` without an upper bound. Malformed or malicious headers can force extreme allocation attempts.

## Scope
In scope:
- Define and enforce deterministic maximum framed `Content-Length` in `kamn-mcp-server` main stdio loop.
- Fail closed with deterministic stderr marker and exit code `2` when bound is exceeded.
- Add tests for boundary acceptance and oversized rejection behavior.

Out of scope:
- Transport-level rate limiting.
- Dynamic runtime config for the cap.
- Refactors outside framed-stdio content-length handling.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Framed requests with `Content-Length` above the max bound are rejected before payload allocation/read.
- AC-2: Rejection path emits deterministic error marker and exits with code `2`.
- AC-3: Valid framed requests at/under bound continue to function.

## Conformance Cases
- C-01 (Unit, AC-1): boundary value (`max`) is accepted by content-length guard.
- C-02 (Unit, AC-1): `max + 1` is rejected by content-length guard.
- C-03 (Integration, AC-2): spawned MCP server receives oversized framed header and exits `2` with deterministic marker.
- C-04 (Regression, AC-3): persistent stdio framed session contract remains green for normal request sizes.

## Success Metrics / Observable Signals
- Oversized framed requests fail before any payload `Vec` allocation for declared size.
- `cargo test -p kamn-mcp-server --test main_stdio_persistent_contract` covers oversized rejection.
- Existing stdio framed happy-path test still passes.
