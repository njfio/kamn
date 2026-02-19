# Issue #4136 Plan

- Issue: #4136
- Status: Reviewed

## Approach
1. Add a shared test helper module for deterministic proptest config and lifecycle invariant helpers.
2. Add helper contract tests validating seed configurability and transition legality helpers.
3. Refactor existing task/escrow and peer proptest suites to use shared helpers.
4. Update runtime state-model docs with helper references.
5. Run targeted suites, fmt, clippy, and shell guardrails.

## Risks and Mitigations
- Risk level: medium
- Risks:
  - Refactor could unintentionally alter deterministic seed behavior.
  - Shared helper signatures could be too narrow and force future duplication.
- Mitigations:
  - Add explicit helper contract tests before refactoring suites.
  - Keep helper APIs generic for source-file, seed env key, and transition checks.

## Interface Contract
- Test-surface helper API only.
- No production API changes.

## ADR
- Not required (test architecture consolidation).
