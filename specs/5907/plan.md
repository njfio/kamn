# Plan: Issue #5907 - Nonce Overflow Fail-Closed in kamn-agent-lib

## Approach
1. Introduce explicit overflow signaling from `NonceTracker` when `current == u64::MAX`.
2. Update `AgentLib::next_nonce` to propagate this as `AgentLibError::InvalidInput` with stable reason text.
3. Add unit tests for normal monotonic behavior and overflow behavior in `nonce.rs`.
4. Add integration/regression tests in `kamn-agent-lib` verifying API-level fail-closed behavior when nonce is exhausted.

## Affected Modules
- `crates/kamn-agent-lib/src/nonce.rs`
- `crates/kamn-agent-lib/src/lib.rs`

## Risks / Mitigations
- Risk: API signature changes may require many downstream updates.
  - Mitigation: keep external `AgentLib` API unchanged; only internal nonce tracker signaling changes.
- Risk: error drift causes unstable contracts.
  - Mitigation: use deterministic existing `AgentLibError::InvalidInput` mapping and lock with regression tests.

## Interfaces / Contracts
- `NonceTracker` gains fail-closed overflow signaling semantics.
- `AgentLib` request methods continue returning `Result<_, AgentLibError>`, now including nonce-overflow mapping.

## ADR
- Not required (no dependency/protocol/architecture boundary change).
