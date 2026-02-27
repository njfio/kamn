# Spec: Issue 6215 - Clarify Payload Hash Value Semantics in Runtime Commit Identity

- Issue: #6215
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P2
- Area: docs

## Problem Statement

Runtime commit identity previously had ambiguous `payload_hash` semantics from
historical length-based composition. Current behavior is value-based, but
comments/tests should make this explicit to prevent regression and confusion.

## Scope

In scope:
1. Clarify code-level semantics that payload hash value bytes drive identity fields.
2. Add regression coverage for idempotency key value-based behavior.

Out of scope:
1. Runtime wire-format changes.
2. Further hash algorithm redesign.

## Acceptance Criteria

### AC-1 Value-Based Semantics Documented
Given runtime commit identity helpers,
When reading source,
Then comments explicitly state payload hash VALUE bytes are used (not length).

### AC-2 Value-Based Idempotency Coverage
Given two equal-length distinct payload hash values,
When idempotency key helper runs,
Then generated keys differ.

## Conformance Cases

- C-01 (AC-1, Unit): `runtime_request_identity_policy::tests::regression_issue_6202_commit_id_uses_payload_hash_value_component`
- C-02 (AC-2, Unit): `runtime_request_identity_policy::tests::regression_issue_6215_idempotency_key_uses_payload_hash_value_component`

