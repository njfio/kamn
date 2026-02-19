# Issue #3793 Plan

- Issue: #3793
- Status: Reviewed

## Approach
1. Add deterministic reconnect pacing helper to notifications consumer reconnect loop.
2. Keep reconnect attempt budget exhaustion behavior deterministic and unchanged for terminal outcomes.
3. Add/extend tests for pacing schedule, reconnect-loop behavior, and bounded delay budget.
4. Add runtime architecture doc markers for reconnect pacing and pin them with docs-contract assertions.
5. Run targeted tests + lint + shell guardrails, then merge and close issue.

## Risks and Mitigations
- Risk level: medium
- Risks:
  - New reconnect delays can introduce flaky time-based tests.
  - Overly aggressive delays can regress runtime responsiveness.
- Mitigations:
  - Use deterministic bounded backoff with small cap.
  - Keep timing assertions tolerant and bounded to avoid flaky scheduling noise.
  - Add explicit performance-bound test for reconnect pacing path.

## Interface Contract
- No API/protocol changes; internal reconnect-loop behavior only.

## ADR
- Not required.
