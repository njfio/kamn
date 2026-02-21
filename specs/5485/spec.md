# Issue #5485 Spec - R49 Artifact Post-Publication Revalidation Refresh

- Status: Implemented
- Issue: #5485
- Parent: #3812
- Milestone: R50.8 R49 review artifact post-publication baseline revalidation

## Problem Statement
`docs/review/gaps-and-issues-r49.md` captures publication-time baseline markers that no longer match current post-publication state (issue #5469 and milestone #98 are closed; remote branch count changed), which can mislead operational consumers reading the artifact without external cross-checking.

## Scope
In scope:
- Re-run deterministic baseline evidence commands.
- Add post-publication revalidation section with exact current markers.
- Preserve original publication snapshot context.

Out of scope:
- Any runtime/product logic change.
- New CI workflow/script behavior.

## Acceptance Criteria
- AC-1: Artifact documents post-publication revalidation markers for open issues, open milestones, and remote branch heads with exact current values.
- AC-2: Artifact includes deterministic revalidation evidence commands and captured outputs for refreshed markers.
- AC-3: Validation commands (ignored-test drift checker, fmt, strict clippy, targeted docs contract checks) remain green after update.

## Conformance Cases
- C-01 (Docs/Structural, AC-1): `docs/review/gaps-and-issues-r49.md` contains explicit post-publication marker block with `open_issue_count=0`, `open_milestone_count=0`, and current branch-head count.
- C-02 (Docs/Evidence, AC-2): artifact includes command/output snippets proving refreshed marker values.
- C-03 (Regression, AC-3): baseline validation commands run successfully and no docs-contract regressions are introduced.

## Success Metrics / Observable Signals
- Consumers can distinguish publication snapshot from refreshed post-publication state directly within the artifact.
- Deterministic marker block matches current verified command outputs.
