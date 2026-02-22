# Spec: Issue #5791 — Enforce R54+ Governance-Remediation Commit-Budget Docs-Contract

- Issue: #5791
- Milestone: `r52-e2e-live-runtime-integration-hardening`
- Priority: P2
- Status: Implemented
- Last Updated: 2026-02-22

## Problem Statement
R53 recommendation #5 defines a governance-remediation budget target (max 5 commits per remediation item), but there is no fail-closed docs-contract requiring future review artifacts to publish and validate this metric.

## Scope
- Add policy baseline file defining governance-remediation budget contract parameters.
- Extend review docs-contract test coverage so review artifacts for release >=54 must include governance-remediation budget markers and consistency math.
- Preserve existing R53 freeze and post-publication moratorium checks.

## Out of Scope
- Editing historical R52/R53 review docs.
- Runtime/module/API changes.
- Shell/workflow/template changes.

## Acceptance Criteria

### AC-1: R54+ review docs require governance-remediation budget markers
Given review artifacts with release number >=54,
When docs-contract tests scan marker keys,
Then required governance-remediation budget markers are present.

### AC-2: Budget math and status are fail-closed
Given R54+ governance-remediation budget markers,
When tests parse counts and ratios,
Then `commits_per_item ~= commit_count / item_count` and status reflects whether value is within policy max.

### AC-3: Existing R53 contract lanes remain passing
Given new budget checks,
When R53 docs-contract and spec-cap guard lanes run,
Then they remain green.

## Conformance Cases

| ID | AC | Tier | Case |
|---|---|---|---|
| C-01 | AC-1 | Functional | R54+ review docs include required budget marker family. |
| C-02 | AC-2 | Regression | RED on missing policy/budget markers; GREEN after policy integration. |
| C-03 | AC-3 | Integration | `review_r53_docs_contract` + `review_r50_spec_volume_remediation_docs_contract` pass. |

## Success Metrics / Observable Signals
- Targeted RED->GREEN evidence for governance-remediation budget check.
- Integrated docs-contract lanes pass with no regressions.
- Spec-directory cap preserved.
