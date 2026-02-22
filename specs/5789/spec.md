# Spec: Issue #5789 — Enforce R54+ Post-Publication Reconciliation Moratorium Docs-Contract

- Issue: #5789
- Milestone: `r52-e2e-live-runtime-integration-hardening`
- Priority: P2
- Status: Implemented
- Last Updated: 2026-02-22

## Problem Statement
R53 explicitly recommends a moratorium on post-publication reconciliation sections and marker-about-marker loops. Current contracts do not fail closed if future review artifacts reintroduce `Post-Publication` appendix sections or `_post_publication_` marker keys.

## Scope
- Extend docs-contract coverage to scan release review files `docs/review/gaps-and-issues-r*.md` for releases `>=54`.
- Fail if these future review files contain post-publication appendix headings or `_post_publication_` marker keys.
- Preserve all existing R53 snapshot/freeze contract behavior.

## Out of Scope
- Editing historical R52/R53 review artifacts.
- Runtime/API/module changes.
- Shell/workflow/template surface changes.

## Acceptance Criteria

### AC-1: R54+ review docs reject post-publication appendix headings
Given review docs with release number >=54,
When docs-contract tests scan section headings,
Then no heading may include `Post-Publication`.

### AC-2: R54+ review docs reject post-publication marker keys
Given review docs with release number >=54,
When docs-contract tests scan marker lines,
Then no marker key may contain `_post_publication_`.

### AC-3: Existing R53 contract lanes remain green
Given moratorium checks are added,
When R53 docs-contract and spec-volume cap lanes run,
Then they remain passing.

## Conformance Cases

| ID | AC | Tier | Case |
|---|---|---|---|
| C-01 | AC-1 | Functional | Moratorium test fails if any R54+ heading contains `Post-Publication`. |
| C-02 | AC-2 | Regression | Moratorium test fails if any R54+ marker key contains `_post_publication_`. |
| C-03 | AC-3 | Integration | Existing `review_r53_docs_contract` and `review_r50_spec_volume_remediation_docs_contract` pass unchanged. |

## Success Metrics / Observable Signals
- New moratorium test passes in `review_r53_docs_contract`.
- Existing R53 freeze/marker and R50 spec-cap lanes pass.
- Spec-dir cap remains preserved.
