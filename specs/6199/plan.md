# Plan: Issue 6199 - MCP Framed Input Must Enforce Max Content-Length

- Issue: #6199
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Introduce `MAX_FRAMED_CONTENT_LENGTH_BYTES` constant in `main.rs`.
2. Add helper to validate parsed content-length against cap.
3. Invoke validation in framed request branch before `Vec` allocation.
4. Add unit regressions for accepted/rejected boundary values.
5. Re-run existing persistent stdio contract test to verify no behavior drift.

## Affected Modules

- `crates/kamn-mcp-server/src/main.rs`
- `crates/kamn-mcp-server/tests/main_stdio_persistent_contract.rs`

## Risks and Mitigations

1. Risk: cap too small for expected MCP payloads.
   - Mitigation: choose conservative 1 MiB default and document constant.
2. Risk: framed parsing exit path changes observability.
   - Mitigation: deterministic startup-loop error path and unit assertions on helper output.
