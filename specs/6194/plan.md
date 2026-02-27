# Plan: Issue 6194 - Message Expiration Must Use Lifecycle Transition Contract

- Issue: #6194
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Replace direct `apply_status(..., Expired)` calls in expiration APIs with `transition(..., Expired)`.
2. Extend `is_valid_transition` to include explicit expiration edges from overdue-eligible states.
3. Align expiration eligibility helper semantics with new transition contract.
4. Add regression tests for validated overdue expiration and sweep behavior.

## Affected Modules

- `crates/kamn-core/src/message_lifecycle.rs`

## Risks and Mitigations

1. Risk: transition map changes could alter lifecycle behavior unexpectedly.
   - Mitigation: focused unit tests for expiration sweep and validated-message expiration.
2. Risk: broad kamn-core test matrix is expensive in constrained environments.
   - Mitigation: execute scoped `--lib` tests covering changed lifecycle paths.
