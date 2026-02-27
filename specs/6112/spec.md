# Spec: Issue #6112 - Snapshot journal helper deduplication

Status: Reviewed
Issue: #6112
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
Snapshot/journal helper logic is duplicated across multiple modules (`message_lifecycle.rs`, `channel_models.rs`, `task_operations.rs`, plus related stores). The duplicated helpers increase drift risk and make bugfixes expensive.

## Scope
In scope:
- Extract shared helper logic for journal hex encoding/decoding and journal replay line handling into a common module.
- Refactor the targeted snapshot stores to use shared helpers while preserving existing behavior.
- Add regression tests that lock helper behavior.

Out of scope:
- Broad redesign of store formats.
- Changing serialized wire format for existing snapshot payloads.
- Refactoring unrelated state-machine/business logic.

## Acceptance Criteria
- AC-1: Shared helper module replaces duplicated journal hex helpers in targeted snapshot stores.
- AC-2: Shared helper behavior is covered by unit/regression tests (roundtrip + corrupt tail handling primitives).
- AC-3: Existing snapshot store tests for message/channel/task flows remain green, proving no behavioral regression.

## Conformance Cases
- C-01 (AC-1): `message_lifecycle`, `channel_models`, and `task_operations` use the same shared journal hex encode/decode implementation.
- C-02 (AC-2): Shared helper tests verify hex roundtrip, odd-length rejection, and invalid-hex rejection.
- C-03 (AC-3): Targeted snapshot store contract tests pass after refactor.

## Success Metrics
- `cargo test -p kamn-core message_lifecycle::tests::`
- `cargo test -p kamn-core channel_models::tests::`
- `cargo test -p kamn-core task_operations::tests::`
