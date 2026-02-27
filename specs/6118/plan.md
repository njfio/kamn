# Plan: Issue #6118

## Approach
1. Add RED integration coverage for missing/unreadable key-file startup failure and update persistent-session test to use real temp key-file input.
2. Add startup helper(s) in `main.rs` to load key-file content and construct `AgentIdentity` via `from_did_and_signing_key`.
3. Replace `KamnAgentHandle::connect(...)` with `KamnAgentHandle::with_identity(...)` in MCP startup.
4. Add unit tests for key-file loader boundary cases (valid, empty, unreadable).
5. Run targeted MCP tests, fmt, and clippy.

## Affected Modules
- `crates/kamn-mcp-server/src/main.rs`
- `crates/kamn-mcp-server/tests/main_stdio_persistent_contract.rs`
- `specs/6118/spec.md`
- `specs/6118/plan.md`
- `specs/6118/tasks.md`

## Risks / Mitigations
- Risk: startup fails in test/dev due missing fixture key files.
  Mitigation: integration tests create deterministic temp key files before spawn.
- Risk: malformed DID derivation from agent name.
  Mitigation: explicit normalization + validation helper with deterministic error text.

## Interfaces / Contracts
- New internal startup helper in MCP main:
  - load signing key from file path
  - normalize agent name
  - construct `AgentIdentity` and wire into `KamnAgentHandle::with_identity`
