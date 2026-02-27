# Plan: Issue #6115

## Approach
1. Update expiry code paths to invoke `transition(message_id, MessageStatus::Expired)`.
2. Expand transition policy to include explicit expirable-status -> `Expired` edges.
3. Replace/extend tests to validate overdue expiry for `Validated` state and history correctness.
4. Run targeted lifecycle tests, then full `kamn-core` library tests.

## Affected Modules
- `crates/kamn-core/src/message_lifecycle.rs`

## Risks
- Risk: behavior change for validated/rejected lifecycle states could impact downstream expectations.
  - Mitigation: keep change scoped, assert history and status transitions in regression tests.

## Interfaces/Contracts
- Internal lifecycle policy only; no public API signature changes.
