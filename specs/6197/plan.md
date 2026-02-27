# Plan: Issue 6197 - MCP Server Must Consume `--key-file` Identity Material

- Issue: #6197
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Add startup helpers in `crates/kamn-mcp-server/src/main.rs`:
   - load key content from file path,
   - normalize `agent_name` into a DID suffix,
   - build explicit `AgentIdentity::from_did_and_signing_key`.
2. Replace `KamnAgentHandle::connect(...)` with `KamnAgentHandle::with_identity(...)`.
3. Keep request/response runtime unchanged after handle creation.
4. Add focused unit tests for key-file loading and DID identity assembly helpers.

## Affected Modules

- `crates/kamn-mcp-server/src/main.rs`
- `crates/kamn-mcp-server/tests/main_stdio_persistent_contract.rs`

## Risks and Mitigations

1. Risk: incompatible key-file formatting (whitespace/newline).
   - Mitigation: trim file content before validation.
2. Risk: agent-name normalization drift.
   - Mitigation: enforce same `[a-zA-Z0-9_-]` contract in startup helper tests.
