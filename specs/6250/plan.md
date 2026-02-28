# Issue 6250 Plan

## Approach
1. Capture baseline shell/rust LOC metrics and identify highest-churn script lanes.
2. Select one high-frequency lane for migration to Rust-owned tests/utilities.
3. Implement RED tests and then migrate lane logic.
4. Add/strengthen ratio measurement + non-regression gate in CI.
5. Recompute metrics and verify ratio target, then document actual deltas.

## Affected Modules
- `scripts/ci/*` selected lane wrappers
- Rust integration tests/utilities for migrated lane
- CI gate scripts/workflows that measure shell/rust surface ratio
- `docs/planning/r59-followup.md`
- `specs/6250/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: ratio target is missed due to limited migration scope.
  - Mitigation: pick a high-LOC high-churn lane with measurable impact.
- Risk: CI gate instability from measurement variability.
  - Mitigation: deterministic file selection rules and reason-coded fail-closed behavior.
- Risk: migrated lane behavior diverges from legacy script behavior.
  - Mitigation: regression tests comparing old/new expected outputs and semantics.

## Interfaces
- CI governance scripts/workflows and Rust test harnesses.
- No runtime external API changes.
