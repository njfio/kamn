# Spec: Issue #5973 - Epic: R66 Residual Structural Gap Closure from R57 Review

- Issue: #5973
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-25)
- Type: epic
- Priority: P1
- Area: program
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25

## Problem Statement
R57 identified structural gaps. Current mainline already closed several high-risk items, but residual gaps remain and need explicit closure with deterministic non-regression gates.

## Scope
In scope:
- Remove deterministic baseline transport auth from production paths.
- Rebalance governance-vs-behavioral assurance with enforceable telemetry gates.
- Guard previously high-risk R57 closures (persistence/relay/live E2E) against regression.

Out of scope:
- New product feature expansion unrelated to residual R57 closure.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Production transport auth paths no longer rely on deterministic baseline signatures.
- AC-2: Governance/runtime assurance coupling has explicit CI telemetry and enforceable thresholds.
- AC-3: Prior high-risk R57 closures are guarded by deterministic non-regression checks.

## Conformance Cases
- C-01 (Conformance, AC-1): Verify production SDK/agent transport requests require cryptographic signatures end-to-end.
- C-02 (Conformance, AC-2): Verify governance/runtime ratio telemetry is emitted and gate-enforced in CI.
- C-03 (Conformance, AC-3): Verify persistence/relay/live-E2E guard checks fail closed on drift.

## Success Metrics / Observable Signals
- All child stories/tasks complete with AC mapping and passing CI.
- No deterministic baseline signature usage remains in production request paths.
- CI emits stable non-regression evidence markers for prior high-risk gaps.
