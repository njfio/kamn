# Plan: Issue 6215 - Clarify Payload Hash Value Semantics in Runtime Commit Identity

- Issue: #6215
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Add explicit inline comments in runtime identity helpers stating value-based payload component behavior.
2. Add regression for idempotency key behavior across distinct equal-length payload values.
3. Run scoped format/lint/tests for `kamn-kolme`.

## Affected Modules

- `crates/kamn-kolme/src/runtime_request_identity_policy.rs`

## Risks and Mitigations

1. Risk: no-op docs-only change without behavioral guard.
   - Mitigation: add regression test for value-based idempotency output.

