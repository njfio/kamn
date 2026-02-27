# Spec: Issue 6202 - Runtime Commit Identity Must Use Payload Hash Value

- Issue: #6202
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P1
- Area: backend

## Problem Statement

Kolme runtime commit identity currently derives from `payload_hash.len()` so distinct hash
values with the same length collide.

## Scope

In scope:
1. Derive runtime commit identifier component from payload-hash value bytes.
2. Apply same value-based component to idempotency key derivation.
3. Update regressions to prove equal-length distinct payload hashes no longer collide.

Out of scope:
1. Provider receipt format redesign.
2. New external hashing dependencies.

## Acceptance Criteria

### AC-1 Commit ID Uses Payload Value
Given two payload hash strings with equal length but different values,
When deterministic commit IDs are generated,
Then resulting IDs are distinct.

### AC-2 Idempotency Key Uses Payload Value
Given runtime commit idempotency derivation,
When payload hash changes,
Then key component reflects payload value bytes instead of length.

### AC-3 Contract Tests Prevent Regression
Given identity policy contract tests,
When run,
Then they fail if value-based component regresses to length-based behavior.

## Conformance Cases

- C-01 (AC-1, Unit): `runtime_request_identity_policy::tests::regression_issue_6202_commit_id_uses_payload_hash_value_component`
- C-02 (AC-2, Functional): `runtime_request_identity_policy_contracts::unit_runtime_request_identity_policy_idempotency_key_contract`
- C-03 (AC-3, Functional): `runtime_request_identity_policy_contracts::regression_issue_6202_runtime_request_identity_policy_payload_component_uses_value_not_length`
