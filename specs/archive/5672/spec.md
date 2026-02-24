# Spec: #5672 Activate Remaining CLI Core Message/Task Operations

- Issue: #5672
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`kamn-cli` still leaves several core operations unsupported even though `kamn-agent-lib` already supports them.

## Scope
### In Scope
- Implement: `register`, `send-message`, `create-channel`, `query-message`, `create-task` command modules.
- Add deterministic argument-validation behavior for required payload/id args.
- Add conformance tests for each command through `dispatch()`.
- Preserve explicit unsupported responses for `accept-task`, `complete-task`, `fund-escrow`, `release-escrow`.

### Out of Scope
- New service API routes.
- Activation of currently unsupported task/escrow command surfaces.

## Acceptance Criteria
### AC-1 Register command
Given CLI register command,
When executed,
Then returns deterministic local DID projection.

### AC-2 Send-message command
Given valid payload arg,
When executed,
Then calls `KamnAgentHandle::send_message` and returns message receipt projection.

### AC-3 Create-channel command
Given valid payload arg,
When executed,
Then calls `KamnAgentHandle::create_channel` and returns channel receipt projection.

### AC-4 Query-message command
Given valid message id arg,
When executed,
Then calls `KamnAgentHandle::query_message` and returns status projection.

### AC-5 Create-task command
Given valid payload arg,
When executed,
Then calls `KamnAgentHandle::create_task` and returns task receipt projection.

### AC-6 Deterministic invalid-input behavior
Given missing required args for supported commands,
When executed,
Then returns deterministic invalid-input errors.

### AC-7 Unsupported regression preservation
Given unsupported commands,
When executed,
Then returns deterministic unsupported responses.

## Conformance Cases
- C-01 (AC-1): register returns DID marker derived from configured agent identity.
- C-02 (AC-2, AC-6): send-message succeeds with payload arg and fails deterministically when missing.
- C-03 (AC-3, AC-6): create-channel succeeds with payload arg and fails deterministically when missing.
- C-04 (AC-4, AC-6): query-message succeeds with message-id arg and fails deterministically when missing.
- C-05 (AC-5, AC-6): create-task succeeds with payload arg and fails deterministically when missing.
- C-06 (AC-7): unsupported command paths remain explicit unsupported responses.

## Success Metrics
- `cargo test -p kamn-cli` passes with added conformance coverage.
- `cargo fmt --all --check` and `cargo clippy -p kamn-cli -- -D warnings` pass.
