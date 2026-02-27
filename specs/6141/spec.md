# Spec: Issue #6141 - Task: [X-03] Run fuzz targets in CI by default policy lane

- Issue: #6141
- Status: Reviewed
- Type: task
- Priority: P1
- Area: qa
- Milestone: `r68-r59-swarm-remediation-and-full-gap-closure`
- Last Updated: 2026-02-27
- Parent: #6102

## Problem Statement
Wire fuzz execution into CI with bounded-cost cadence and evidence artifacts.

## Scope
In scope:
- Implement targeted remediation for `X-03` from `docs/review/gaps-and-issues-r59-swarm.md`.
- Add/adjust conformance, regression, and functional test coverage for the remediated path.
- Update affected documentation and lifecycle artifacts within the same change-set.

Out of scope:
- Unrelated refactors outside `X-03`.
- Unscoped protocol/schema redesign not required by the finding.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: The X-03 gap is remediated with production-safe behavior.
- AC-2: Regression/conformance tests cover the remediation path.
- AC-3: Issue closure includes measurable evidence and linked PR.

## Conformance Cases
- C-01 (Conformance, AC-1): Implemented behavior resolves R59 X-03 with deterministic pass/fail signals.
- C-02 (Regression, AC-2): RED->GREEN test sequence demonstrates failing precondition and passing post-remediation behavior.
- C-03 (Conformance, AC-3): Issue closure references PR, test commands, and measurable outputs tied to acceptance criteria.

## Success Metrics / Observable Signals
- Targeted R59 finding `X-03` no longer appears as unresolved in follow-up review docs.
- Required scoped test commands pass in CI and local verification runs.
- Closure comment includes deterministic evidence links and tier coverage summary.
