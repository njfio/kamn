# Issue #5424 Spec — Merged-Branch Cleanup Wave and Branch Hygiene Re-Baseline

- Status: Reviewed
- Issue: #5424
- Parent: #3812
- Milestone: R27 Program: operational hardening and live validation

## Problem Statement
Branch hygiene drift increases operational overhead and creates avoidable governance noise. We need a deterministic merged-only cleanup wave with evidence-backed before/after counts and refreshed review markers.

## Scope
In scope:
- Measure remote branch count before cleanup.
- Delete only remote branches already merged into `main`.
- Re-measure post-cleanup count and document evidence.
- Update `docs/review/gaps-and-issues-r45.md` branch hygiene markers to the refreshed baseline.

Out of scope:
- Deleting unmerged or active branches.
- Any force push/history rewrite.

## Acceptance Criteria
- AC-1: Cleanup uses merged-only criteria and does not target `main` or `HEAD` symbolic refs.
- AC-2: Post-cleanup remote branch count is at or below 60.
- AC-3: R45 review doc branch hygiene section reflects refreshed counts and evidence command markers.
- AC-4: Spec/plan/tasks artifacts and issue process logs capture the full lifecycle.

## Conformance Cases
- C-01 (Functional, AC-1): candidate branch list is derived from `git branch -r --merged origin/main` excluding `main`/`HEAD`.
- C-02 (Conformance, AC-2): measured post-cleanup `git ls-remote --heads origin | wc -l` is `<=60`.
- C-03 (Regression, AC-3): docs marker assertions for branch hygiene baseline and command evidence pass.
- C-04 (Conformance, AC-4): lifecycle artifacts exist at `specs/5424/{spec,plan,tasks}.md` and issue process log comments include Specify/Plan/Implement/Verify states.

## Success Metrics
- Remote branch count reduced from pre-cleanup baseline to `<=60`.
- Only merged branches are removed.
- R45 review document markers remain internally consistent and evidence-backed.

## AC -> Tests Mapping (initial)
- AC-1: command evidence + cleanup script output in issue/PR.
- AC-2: command evidence + docs marker for post-cleanup count.
- AC-3: new docs contract test under `crates/kamn-core/tests/review_branch_hygiene_docs_contract.rs`.
- AC-4: artifact existence + issue process-log comment evidence.
