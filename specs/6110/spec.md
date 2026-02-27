# Spec: Issue #6110 - Atomic service-api state-file persistence

- Issue: #6110
- Status: Reviewed
- Type: task
- Priority: P0
- Area: backend
- Milestone: `specs/milestones/r68-r59-swarm-remediation-and-full-gap-closure/index.md`
- Last Updated: 2026-02-27
- Parent: #6098

## Problem Statement
Service API JSON persistence currently uses direct `fs::write`, which truncates destination files before writing. A crash/power-loss mid-write can leave state files empty/corrupt and break message-store recovery.

## Scope
In scope:
- Replace direct JSON file writes with atomic temp-file write + `rename` in service-api state IO.
- Apply the same atomic replace contract to relay-spool truncation after drain.
- Preserve existing sqlite backend behavior and error taxonomy prefixes.
- Add regression tests for atomic replacement behavior and temp-file cleanup.

Out of scope:
- WAL/transaction log introduction.
- Cross-process file locking redesign.
- Sqlite storage backend behavior changes.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: JSON state persistence writes via same-directory temp file, fsyncs data, and atomically renames into place.
- AC-2: Relay spool drain truncation uses atomic replace semantics (no in-place truncate path).
- AC-3: Atomic write helper cleans up temp artifacts on failure paths where possible.
- AC-4: Existing sqlite persistence path remains unchanged.

## Conformance Cases
- C-01 (Unit, AC-1): atomic writer replaces existing file content and leaves no temp artifact.
- C-02 (Unit, AC-1/AC-3): `persist_service_api_state_payload` writes JSON payload through atomic helper and does not leave temp files.
- C-03 (Unit, AC-2): relay spool drain empties spool file via atomic replace and leaves no temp files.
- C-04 (Unit, AC-4): sqlite state persistence path remains functional.

## Success Metrics / Observable Signals
- No `fs::write(path, payload)` direct path remains for JSON state persistence.
- State IO tests pass with atomic replacement assertions.
- `cargo test -p kamn-node service_api_endpoint::tests::` passes.
