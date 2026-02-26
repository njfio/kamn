# Spec: Issue #6054 - Enforce real SQLite-backed Service API state persistence

- Issue: #6054
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #6053

## Problem Statement
Service API state persistence accepted `.sqlite` paths but still used JSON file read/write semantics. That made sqlite mode nominal only: state lived in plain file payloads and did not exercise `SqliteStoreBackend` durability or key/value storage contracts.

## Scope
In scope:
- Add deterministic tests proving `.sqlite` state paths persist through `SqliteStoreBackend`.
- Load and persist Service API message store snapshots via sqlite namespace/key entries when sqlite path is configured.
- Ensure relay status projection (`created -> relayed`) works against sqlite-backed state.
- Preserve existing JSON file behavior for non-sqlite state paths.

Out of scope:
- Relay protocol or wire-format changes.
- Service API routing/auth behavior changes.
- Daemon relay forwarding logic changes.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: When `state_file` uses sqlite extension, message-store persistence writes snapshot bytes to sqlite namespace/key entries.
- AC-2: Reopening a sqlite-backed message store recovers previously written messages and metadata.
- AC-3: `project_service_api_relayed_message_statuses` promotes created messages to relayed for sqlite-backed state.
- AC-4: Existing JSON state-file projection and message-store contracts remain green.

## Conformance Cases
- C-01 (Integration, AC-1): sqlite-backed message creation yields a readable sqlite namespace/key snapshot row.
- C-02 (Functional, AC-2): reopened sqlite-backed message store returns the created message with expected sender/recipient/status.
- C-03 (Conformance, AC-3): sqlite-backed relay projection updates persisted snapshot status to `relayed`.
- C-04 (Regression, AC-4): existing JSON relay projection tests remain green.

## Success Metrics / Observable Signals
- SQLite-backed tests fail before backend wiring and pass after wiring.
- `cargo test -p kamn-node sqlite_state_backend` passes with sqlite namespace/key assertions.
- Existing relay projection JSON-path tests remain green.
