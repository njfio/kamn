# Plan: Issue 6202 - Runtime Commit Identity Must Use Payload Hash Value

- Issue: #6202
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Update `runtime_request_identity_policy` derivation functions:
   - `deterministic_runtime_commit_id`,
   - `deterministic_runtime_commit_idempotency_key`.
2. Add local deterministic hex encoding helper for payload hash bytes.
3. Update affected unit and contract tests to assert value-based uniqueness.

## Affected Modules

- `crates/kamn-kolme/src/runtime_request_identity_policy.rs`
- `crates/kamn-kolme/tests/runtime_request_identity_policy_contracts.rs`

## Risks and Mitigations

1. Risk: commit-id format drift for consumers assuming length suffix.
   - Mitigation: deterministic format retained; only suffix semantics changed and covered by tests.
2. Risk: delimiter ambiguity if raw payload hash is embedded.
   - Mitigation: hex-encode payload hash bytes before embedding.
