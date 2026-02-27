# Spec: Issue 6197 - MCP Server Must Consume `--key-file` Identity Material

- Issue: #6197
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P1
- Area: backend

## Problem Statement

`kamn-mcp-server` parses `--key-file` but discards it, so operators provide a required
argument that has no runtime effect.

## Scope

In scope:
1. Load signing-key material from `--key-file`.
2. Build explicit `AgentIdentity` from loaded key material.
3. Fail startup deterministically when key material is unreadable/empty.

Out of scope:
1. New encrypted key formats.
2. Key-rotation protocols.

## Acceptance Criteria

### AC-1 Key File Is Runtime Binding
Given a valid `--key-file`,
When `kamn-mcp-server` starts,
Then the runtime identity is constructed from key-file content.

### AC-2 Invalid Key File Fails Closed
Given an unreadable or empty key-file,
When startup initializes identity,
Then startup exits with a deterministic error.

### AC-3 Persistent Session Behavior Remains Stable
Given a valid key-file and framed stdio requests,
When multiple requests are processed in one session,
Then framed response behavior remains unchanged.

## Conformance Cases

- C-01 (AC-1, Unit): `main::tests::regression_issue_6197_load_signing_key_from_file_consumes_key_material`
- C-02 (AC-2, Unit): `main::tests::regression_issue_6197_load_signing_key_from_file_rejects_empty_content`
- C-03 (AC-3, Integration): `main_stdio_persistent_contract::spec_c10_main_stdio_session_processes_multiple_framed_requests_without_eof`
