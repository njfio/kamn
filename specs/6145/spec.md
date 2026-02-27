# Spec: Issue #6145 - Task: [X-07] Wire multi-node P2P transport into message delivery

- Issue: #6145
- Status: Reviewed
- Type: task
- Priority: P1
- Area: networking
- Milestone: `r68-r59-swarm-remediation-and-full-gap-closure`
- Last Updated: 2026-02-27
- Parent: #6099

## Problem Statement
Close remaining distributed coordination gap by integrating P2P transport into delivery path.

## Scope
In scope:
- Implement targeted remediation for `X-07` from `docs/review/gaps-and-issues-r59-swarm.md`.
- Add/adjust conformance, regression, and functional test coverage for the remediated path.
- Update affected documentation and lifecycle artifacts within the same change-set.

Out of scope:
- Unrelated refactors outside `X-07`.
- Unscoped protocol/schema redesign not required by the finding.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: The X-07 gap is remediated with production-safe behavior.
- AC-2: Regression/conformance tests cover the remediation path.
- AC-3: Issue closure includes measurable evidence and linked PR.

## Conformance Cases
- C-01 (Conformance, AC-1): Implemented behavior resolves R59 X-07 with deterministic pass/fail signals.
- C-02 (Regression, AC-2): RED->GREEN test sequence demonstrates failing precondition and passing post-remediation behavior.
- C-03 (Conformance, AC-3): Issue closure references PR, test commands, and measurable outputs tied to acceptance criteria.

## Success Metrics / Observable Signals
- Targeted R59 finding `X-07` no longer appears as unresolved in follow-up review docs.
- Required scoped test commands pass in CI and local verification runs.
- Closure comment includes deterministic evidence links and tier coverage summary.
