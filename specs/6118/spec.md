# Spec: Issue #6118 - Wire MCP `--key-file` to identity loading

- Issue: #6118
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r68-r59-swarm-remediation-and-full-gap-closure/index.md`
- Last Updated: 2026-02-27
- Parent: #6100

## Problem Statement
`kamn-mcp-server` currently parses `--key-file` but discards it (`let _ = config.key_file.as_str();`), so required key-file input has no runtime effect.

## Scope
In scope:
- Load signing-key material from `--key-file` during MCP server startup.
- Build explicit `AgentIdentity` from DID + signing key and pass it into `KamnAgentHandle::with_identity`.
- Fail closed with deterministic startup error when key-file loading/parsing fails.
- Update integration tests to supply real temp key-file content.

Out of scope:
- Encrypted key-file formats.
- Multi-key keychain management.
- Changes to `kamn-agent-lib` identity model outside this startup path.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: MCP server startup reads `--key-file` and uses it to construct agent identity instead of ignoring the argument.
- AC-2: Invalid/missing key-file content causes deterministic connect/config error and non-zero exit.
- AC-3: Existing framed stdio session behavior remains functional when valid key-file content is provided.

## Conformance Cases
- C-01 (Unit, AC-1): identity loader from key-file returns identity using normalized agent DID and file-provided signing key.
- C-02 (Unit, AC-2): missing or empty key-file content is rejected with deterministic reason.
- C-03 (Integration, AC-3): persistent stdio session test passes with temp key-file fixture.
- C-04 (Regression, AC-2): startup fails closed when key-file path is unreadable.

## Success Metrics / Observable Signals
- `--key-file` no longer appears as a dead argument in `main.rs`.
- MCP binary startup path is covered by tests using an actual key-file fixture.
- `cargo test -p kamn-mcp-server --test main_stdio_persistent_contract` remains green.
