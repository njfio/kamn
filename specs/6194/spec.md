# Spec: Issue 6194 - Message Expiration Must Use Lifecycle Transition Contract

- Issue: #6194
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P1
- Area: backend

## Problem Statement

Message expiration paths used direct status mutation (`apply_status`) instead of the public
transition validator, creating lifecycle asymmetry and bypass risk.

## Scope

In scope:
1. Route expiration state changes through `transition()` validation.
2. Define explicit `-> Expired` transition edges for overdue lifecycle states.
3. Add regression coverage for validated-message expiration behavior.

Out of scope:
1. Reworking the full lifecycle taxonomy.
2. Introducing time-source abstraction changes.

## Acceptance Criteria

### AC-1 No Expiration Bypass
Given overdue message expiration paths,
When status is moved to `Expired`,
Then the transition validator is used instead of direct status mutation.

### AC-2 Deterministic Transition Rules
Given lifecycle states eligible for overdue expiration,
When expiration occurs,
Then transitions are explicitly represented in `is_valid_transition`.

### AC-3 Regression Coverage
Given a validated overdue message,
When expiration is evaluated,
Then the message transitions to `Expired` and tests fail if behavior regresses.

## Conformance Cases

- C-01 (AC-1, Unit): `expire_message_if_overdue` and `expire_overdue_messages` call `transition(..., Expired)`.
- C-02 (AC-2, Unit): `is_valid_transition` includes explicit `-> Expired` edges for overdue-expirable states.
- C-03 (AC-3, Unit): `message_lifecycle::tests::regression_issue_6194_expire_message_if_overdue_transitions_validated_message`.
