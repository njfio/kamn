# Spec: Issue #6115 - Message Lifecycle Expiry Transition Enforcement

Status: Reviewed
Issue: #6115
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
`MessageLifecycleStore::expire_message_if_overdue` and `MessageLifecycleStore::expire_overdue_messages` write `Expired` status via `apply_status` directly, bypassing `transition` validation. This allows lifecycle history edges that are not validated by the transition policy and diverges from the normal transition contract.

## Scope
In scope:
- Route expiry operations through the same transition validator used by other lifecycle state changes.
- Define explicit transition edges for statuses that are permitted to expire by overdue sweep.
- Update lifecycle tests to lock the new expiry transition behavior.

Out of scope:
- Broader lifecycle redesign.
- Changes outside `crates/kamn-core/src/message_lifecycle.rs`.

## Acceptance Criteria
- AC-1: Expiry operations (`expire_message_if_overdue`, `expire_overdue_messages`) no longer bypass transition validation.
- AC-2: Overdue `Validated` messages are eligible for expiry under explicit transition policy.
- AC-3: Regression/conformance tests cover expiry transitions and fail closed on invalid transition edges.

## Conformance Cases
- C-01 (AC-1): Expiry functions call `transition(..., Expired)` rather than direct status application.
- C-02 (AC-2): A message in `Validated` state expires when overdue and records `Validated -> Expired` in history.
- C-03 (AC-3): Existing lifecycle transition tests continue passing with updated transition graph and expiry behavior.

## Success Metrics
- `cargo test -p kamn-core message_lifecycle::tests::`
- `cargo test -p kamn-core --lib`
