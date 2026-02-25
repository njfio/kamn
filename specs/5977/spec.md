# Spec: Issue #5977 - Task: Replace SDK/agent-lib baseline transport signatures with cryptographic profile

- Issue: #5977
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-25)
- Type: task
- Priority: P1
- Area: sdk
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25
- Parent: #5974

## Problem Statement
Production transport request signing still permits deterministic baseline construction in SDK/agent-lib paths.

## Scope
In scope:
- Replace deterministic production request signature construction.
- Ensure service verification contract accepts cryptographic profile and rejects deterministic fallback by default.

Out of scope:
- Kolme runtime commit signature flow.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Production SDK/agent-lib request signing uses cryptographic profile.
- AC-2: Baseline deterministic signing is removed from production request call paths.
- AC-3: Negative auth cases (tamper/replay/wrong-key/malformed) fail closed in integration tests.

## Conformance Cases
- C-01 (Functional, AC-1): Signed service request validates with cryptographic verifier.
- C-02 (Regression, AC-2): Deterministic baseline signature injection is rejected in production mode.
- C-03 (Integration, AC-3): Auth failure matrix remains deterministic and reason-coded.

## Success Metrics / Observable Signals
- `rg` check shows no deterministic baseline signature constructor usage in production SDK/agent-lib send paths.
- Scoped SDK/node auth tests pass.
