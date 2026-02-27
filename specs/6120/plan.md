# Plan: Issue #6120

## Approach
1. Add RED tests for oversized framed content-length rejection in `main_stdio_persistent_contract`.
2. Introduce a deterministic content-length cap and validation helper in `src/main.rs`.
3. Validate final framed `Content-Length` before payload allocation/read.
4. Add unit tests for boundary acceptance/rejection of content-length guard.
5. Run targeted MCP server tests, fmt, and clippy.

## Affected Modules
- `crates/kamn-mcp-server/src/main.rs`
- `crates/kamn-mcp-server/tests/main_stdio_persistent_contract.rs`
- `specs/6120/spec.md`
- `specs/6120/plan.md`
- `specs/6120/tasks.md`

## Risks / Mitigations
- Risk: cap too small for legitimate MCP payloads.
  Mitigation: choose conservative 1 MiB cap suitable for framed JSON-RPC control payloads.
- Risk: regress framed-session behavior.
  Mitigation: keep existing persistent framed integration test unchanged and passing.

## Interfaces / Contracts
- New internal guard in binary main loop:
  - max framed content-length constant
  - validator returning deterministic error text on overflow
