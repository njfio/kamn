# Spec: Issue #6123 - Runtime Commit Identity Uses Payload Value Fingerprint

Status: Reviewed
Issue: #6123
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
`deterministic_runtime_commit_id` and `deterministic_runtime_commit_idempotency_key` in `kamn-kolme` currently encode `payload_hash.len()` rather than payload-hash value content. Equal-length but different payload hashes collide, undermining deterministic identity uniqueness.

## Scope
In scope:
- Replace payload-length component with deterministic payload-value fingerprint component.
- Update unit/contract tests to assert non-collision for distinct payload hash values.
- Update impacted docs/spec artifacts for the behavior change.

Out of scope:
- Cross-crate provider commit-id format redesign.
- Changes outside `kamn-kolme` runtime request identity policy scope.

## Acceptance Criteria
- AC-1: Runtime commit/idempotency identity includes payload-hash value fingerprint, not only length.
- AC-2: Distinct payload-hash values with equal length produce distinct runtime commit IDs.
- AC-3: Contract tests and regressions capture the new non-collision behavior.

## Conformance Cases
- C-01 (AC-1): `deterministic_runtime_commit_id` and idempotency key output include deterministic hex fingerprint of trimmed payload hash value.
- C-02 (AC-2): Equal-length payload hashes (`"abc"`, `"xyz"`) produce different commit IDs.
- C-03 (AC-3): `cargo test -p kamn-kolme --test runtime_request_identity_policy_contracts` passes with updated assertions.

## Success Metrics
- `cargo test -p kamn-kolme --test runtime_request_identity_policy_contracts`
- `cargo test -p kamn-kolme`
