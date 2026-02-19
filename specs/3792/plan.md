# Issue #3792 Plan

- Issue: #3792
- Status: Reviewed

## Approach
1. Add red tests for reconnect terminal taxonomy markers in policy contracts and runtime architecture docs-contracts.
2. Introduce deterministic reconnect terminal taxonomy constants/helpers in `kamn-kolme` notification policy and project them into reconnect exhaustion reason outputs.
3. Update notifications consumer tests to verify marker-bearing terminal reason behavior.
4. Update runtime architecture documentation with explicit reconnect terminal taxonomy markers and table.
5. Run targeted test/lint/guardrail suite, then merge and close issue.

## Risks and Mitigations
- Risk level: low
- Risks:
  - Existing tests/assertions rely on previous reconnect exhaustion reason string format.
  - Marker drift between docs and emitted reason output can cause operator confusion.
- Mitigations:
  - Update and pin contract tests in both policy and docs surfaces.
  - Keep deterministic human-readable reason prefix unchanged while appending explicit markers.

## Interface Contract
- No dependency/API protocol changes; deterministic reason-string contract augmentation only.

## ADR
- Not required.
