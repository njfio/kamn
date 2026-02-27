# Spec: Issue #6127 - Task: [S-04] Dual `next_state`/`transition_between` in task_lifecycle.rs -> single table

- Issue: #6127
- Status: Accepted
- Type: task
- Priority: P2
- Area: backend
- Milestone: `r68-r59-swarm-remediation-and-full-gap-closure`
- Last Updated: 2026-02-27
- Parent: #6101

## Problem Statement
Suggestion from R59 section 8. Impact: Eliminates silent divergence risk

## Scope
In scope:
- Implement targeted remediation for `S-04` from `docs/review/gaps-and-issues-r59-swarm.md`.
- Add/adjust conformance, regression, and functional test coverage for the remediated path.
- Update affected documentation and lifecycle artifacts within the same change-set.

Out of scope:
- Unrelated refactors outside `S-04`.
- Unscoped protocol/schema redesign not required by the finding.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: The S-04 gap is remediated with production-safe behavior.
- AC-2: Regression/conformance tests cover the remediation path.
- AC-3: Issue closure includes measurable evidence and linked PR.

## Conformance Cases
- C-01 (Conformance, AC-1): Implemented behavior resolves R59 S-04 with deterministic pass/fail signals.
- C-02 (Regression, AC-2): RED->GREEN test sequence demonstrates failing precondition and passing post-remediation behavior.
- C-03 (Conformance, AC-3): Issue closure references PR, test commands, and measurable outputs tied to acceptance criteria.

## Success Metrics / Observable Signals
- Targeted R59 finding `S-04` no longer appears as unresolved in follow-up review docs.
- Required scoped test commands pass in CI and local verification runs.
- Closure comment includes deterministic evidence links and tier coverage summary.
