# Spec: Issue #5975 - Story: Governance-to-Behavioral Assurance Rebalance

- Issue: #5975
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-25)
- Type: story
- Priority: P1
- Area: governance
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25
- Parent: #5973

## Problem Statement
Governance/doc-contract tests are structurally heavy and can obscure behavioral assurance signal. Ratio and trend controls need explicit, enforceable guardrails.

## Scope
In scope:
- Deterministic governance/runtime assurance ratio telemetry and policy gate.
- First consolidation wave for high-duplication doc-contract suites.

Out of scope:
- Removing required compliance evidence coverage.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: CI emits deterministic governance/runtime ratio telemetry artifact.
- AC-2: Policy gate enforces non-regression threshold on ratio.
- AC-3: First consolidation wave reduces duplicate doc-contract load without behavior coverage regression.

## Conformance Cases
- C-01 (Conformance, AC-1): Ratio telemetry report generated with stable schema and markers.
- C-02 (Regression, AC-2): Gate fails on threshold breach and passes on compliant baseline.
- C-03 (Functional, AC-3): Consolidated suites preserve required runtime behavioral checks.

## Success Metrics / Observable Signals
- Ratio telemetry appears in CI artifacts with stable schema.
- Redundant doc-contract count decreases while behavior tests remain green.
