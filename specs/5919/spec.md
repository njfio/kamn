# Spec: Issue #5919 - Story: Architecture and Sustainability Remediation

- Issue: #5919
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: story
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent epic: #5915

## Problem Statement
Core architecture coupling, script-surface growth, and duplication reduce reliability and delivery velocity.

## Scope
In scope:
- Decompose kamn-core, reduce shell/python surface, eliminate duplication, and wire data layers.

Out of scope:
- Business feature expansion unrelated to architecture debt reduction.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: A phased crate decomposition plan is executing with measurable module extraction.
- AC-2: Script/code ratio and shell LOC trend downward under enforceable policy gates.
- AC-3: High-impact duplication classes are removed in favor of shared utilities.

## Conformance Cases
- C-01 (Functional, AC-1): Verify A phased crate decomposition plan is executing with measurable module extraction.
- C-02 (Functional, AC-2): Verify Script/code ratio and shell LOC trend downward under enforceable policy gates.
- C-03 (Functional, AC-3): Verify High-impact duplication classes are removed in favor of shared utilities.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: required for touched modules
- Functional: required for behavior paths
- Integration: required where cross-module runtime paths change
- Regression: required for each remediated finding
- Performance: required for hot paths, else justified N/A

## Dependencies
- #5915

