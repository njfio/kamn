# Spec: #5670 Activate `kamn-cli` Execution for Supported Operations

- Issue: #5670
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`kamn-cli` currently returns `UnsupportedOperation` for every command, leaving the CLI surface scaffold-only despite existing `kamn-agent-lib` support for several operations.

## Scope
### In Scope
- Implement real CLI command execution for: `health`, `list-messages`, `verify-proof`.
- Add deterministic argument extraction contract for required positional inputs.
- Introduce shared CLI handle bootstrap helper using endpoint/env defaults.
- Preserve explicit unsupported behavior for unimplemented task/escrow commands.
- Add command-level conformance/regression tests.

### Out of Scope
- `accept-task`, `complete-task`, `fund-escrow`, `release-escrow` activation.
- New service API routes.
- Shell/workflow/template changes.

## Acceptance Criteria
### AC-1 Health command activation
Given a reachable service endpoint,
When `health` command executes,
Then CLI returns deterministic health projection instead of unsupported error.

### AC-2 List-messages command activation
Given a channel id argument,
When `list-messages` command executes,
Then CLI calls `KamnAgentHandle::list_messages` and returns channel/message payload.

### AC-3 Verify-proof command activation
Given `message_id`, `tx_hash`, `block_height`, and `finality` arguments,
When `verify-proof` command executes,
Then CLI calls `KamnAgentHandle::verify_proof` and returns normalized verification payload.

### AC-4 Argument-validation contract
Given missing or malformed required arguments,
When supported commands execute,
Then CLI returns deterministic invalid-input errors.

### AC-5 Unsupported-command regression preservation
Given currently unsupported commands (`accept-task`, `complete-task`, `fund-escrow`, `release-escrow`),
When those commands execute,
Then CLI keeps deterministic unsupported responses.

## Conformance Cases
- C-01 (AC-1): `health` command executes successfully and returns status/runtime fields.
- C-02 (AC-2, AC-4): `list-messages` command succeeds with valid channel id and fails with deterministic invalid-input error when missing.
- C-03 (AC-3, AC-4): `verify-proof` command succeeds with valid args and fails deterministically for malformed `block_height`.
- C-04 (AC-5): unsupported commands remain explicit unsupported responses.

## Success Metrics
- `cargo test -p kamn-cli` passes with new conformance tests.
- `cargo fmt --all --check` and `cargo clippy -p kamn-cli -- -D warnings` pass.
