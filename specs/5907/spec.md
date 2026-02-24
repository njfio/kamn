# Spec: Issue #5907 - Nonce Overflow Fail-Closed in kamn-agent-lib

- Issue: #5907
- Status: Implemented
- Type: task
- Priority: P2
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
`NonceTracker::next_nonce` currently uses saturating arithmetic. At `u64::MAX`, it emits `u64::MAX` repeatedly, enabling duplicate nonce reuse instead of fail-closed behavior.

## Scope
In scope:
- Make nonce advancement fail-closed on overflow.
- Propagate overflow through `kamn-agent-lib` nonce allocation path.
- Add regression tests proving overflow returns an explicit error and does not reuse nonce values.
- Preserve existing monotonic behavior for non-overflow paths.

Out of scope:
- Durable nonce persistence.
- Cross-process nonce leasing/synchronization.

## Acceptance Criteria
### AC-1 Overflow is rejected
Given a nonce tracker at `u64::MAX`,
When requesting the next nonce,
Then the operation returns an explicit overflow error.

### AC-2 No duplicate emission at overflow boundary
Given a tracker advanced to `u64::MAX`,
When next nonce is requested again,
Then no duplicate nonce value is emitted.

### AC-3 Existing monotonic path remains unchanged
Given a tracker below `u64::MAX`,
When requesting next nonce,
Then nonce increments monotonically as before.

### AC-4 Agent-lib request paths fail closed
Given agent-lib request signing that requires nonce allocation,
When nonce tracker is exhausted,
Then operation fails with deterministic nonce-overflow error.

## Conformance Cases
- C-01 (AC-1, Unit): `NonceTracker` returns overflow error at `u64::MAX`.
- C-02 (AC-2, Regression): repeated calls at overflow boundary never emit a duplicate nonce value.
- C-03 (AC-3, Unit): monotonic increment semantics remain unchanged below overflow boundary.
- C-04 (AC-4, Integration): `AgentLib` operations map exhausted nonce tracker to deterministic `AgentLibError`.

## Success Metrics / Observable Signals
- Overflow paths return explicit errors instead of saturating duplicates.
- `kamn-agent-lib` nonce-using APIs fail closed when tracker is exhausted.
- Added unit/regression tests pass and guard against future saturation regressions.
