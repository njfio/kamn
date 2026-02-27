# Spec: Issue #6144 - Task: [X-06] Reduce self-referential governance schema/test overhead

- Issue: #6144
- Status: Reviewed
- Type: task
- Priority: P1
- Area: governance
- Milestone: `r68-r59-swarm-remediation-and-full-gap-closure`
- Last Updated: 2026-02-27
- Parent: #6103

## Problem Statement
Governance contract coverage for review artifacts has accumulated into a self-referential loop:
large marker schema catalogs in `docs/review/README.md` and heavyweight Rust tests validate
governance process mechanics more than product behavior, increasing compile/runtime cost and
maintenance burden.

## Scope
In scope:
- Simplify governance review contract tests to core invariants only.
- Preserve critical safety checks (review freeze/immutability and review activity ratio contract).
- Add conformance/regression coverage for the simplified contract behavior.

Out of scope:
- Changing runtime/product protocol behavior.
- Rewriting historical review documents.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Governance contract tests in `kamn-core` are simplified to essential invariants and no
  longer include large self-referential reconciliation matrices.
- AC-2: Simplified tests still fail closed for missing required core governance markers.
- AC-3: Scoped verification for updated governance contract tests passes with deterministic output.
- AC-4: Targeted governance contract test LOC is materially reduced compared with baseline.

## Conformance Cases
- C-01 (Conformance, AC-1): Target governance contract test modules enforce a reduced set of
  high-value invariants and avoid recursive governance reconciliation checks.
- C-02 (Regression, AC-1/AC-2): Simplified review governance tests reject missing critical markers
  and accept canonical baseline docs.
- C-03 (Functional, AC-3): Updated governance test files compile and pass in scoped `kamn-core`
  contract runs.
- C-04 (Conformance, AC-4): Combined LOC of targeted governance contract tests decreases from
  baseline by a measurable margin.

## Success Metrics / Observable Signals
- LOC across targeted governance contract test files decreases versus baseline.
- Scoped test command for updated governance contracts passes in CI/local verification.
