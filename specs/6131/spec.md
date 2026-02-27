# Spec: Issue #6131 - Bounded anti-spam `seen_message_ids` eviction

Status: Accepted
Issue: #6131
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
`AntiSpamEngine` currently stores every observed `message_id` in an unbounded `HashSet`. Long-running nodes can accumulate unbounded memory, turning duplicate-message protection into a memory growth risk.

## Scope
In scope:
- Add a bounded retention policy for duplicate-message tracking in `AntiSpamEngine`.
- Keep duplicate rejection behavior for retained IDs.
- Define and validate a configurable maximum retained ID count.
- Add unit/regression/conformance coverage for eviction behavior.

Out of scope:
- Persistent duplicate tracking across restarts.
- Changes to sender deposit/rate/suspension policies.
- Cross-crate API redesign beyond required config extension.

## Acceptance Criteria
- AC-1: `AntiSpamEngine` keeps at most `max_seen_message_ids` retained message IDs.
- AC-2: Duplicate IDs are rejected while still retained.
- AC-3: Oldest retained IDs are evicted first; an evicted ID can be accepted again.
- AC-4: Invalid config (`max_seen_message_ids == 0`) fails construction with a typed config error.

## Conformance Cases
- C-01 (AC-1): with `max_seen_message_ids=2`, processing three unique IDs retains only two without growth beyond capacity.
- C-02 (AC-2): with retained ID present, reusing that ID yields `DuplicateMessageId` rejection.
- C-03 (AC-3): after capacity overflow evicts oldest ID, resubmitting that oldest ID is accepted.
- C-04 (AC-4): `AntiSpamEngine::new` rejects `max_seen_message_ids=0` with `InvalidConfig`.

## Success Metrics
- `cargo test -p kamn-runtime-guards anti_spam`
- `cargo test -p kamn-core anti_spam_controls`
- `cargo fmt --check`
- `cargo clippy -p kamn-runtime-guards --tests -- -D warnings`
