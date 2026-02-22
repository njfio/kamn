# Spec: Issue #5793 — Resolve All Unresolved Items in R54 Review

- Issue: #5793
- Milestone: `r52-e2e-live-runtime-integration-hardening`
- Priority: P1
- Status: Implemented
- Last Updated: 2026-02-22

## Problem Statement
`docs/review/gaps-and-issues-r54.md` records unresolved/worsened/recurring items (post-publication marker inflation, governance dominance, branch growth, doc-contract growth, kamn-core module stagnation, and untracked spec-dir contamination). These need concrete, enforced closure rather than narrative-only carryover.

## Scope
- Normalize and track `docs/review/gaps-and-issues-r54.md` with deterministic closure markers for all unresolved R54 items.
- Enforce fail-closed R54 closure marker invariants in existing review docs-contract test coverage.
- Fix spec-volume non-regression counting semantics to use tracked `specs/` directories only.
- Preserve spec-dir cap and milestone lifecycle records in the same delivery.

## Out of Scope
- Large cross-crate feature expansion.
- CI/workflow/template changes.
- Historical rewrite of R50–R53 snapshot markers.

## Acceptance Criteria

### AC-1: R54 unresolved-item closure markers are complete and explicit
Given `docs/review/gaps-and-issues-r54.md`,
When marker keys are parsed,
Then all six unresolved items are represented by closure markers and global closure status is `all_resolved`.

### AC-2: Post-publication inflation gap is closed with moratorium-compliant artifact shape
Given R54 review content,
When moratorium checks and R54 closure checks run,
Then no disallowed post-publication heading pattern remains and marker-inflation closure markers are internally consistent.

### AC-3: Governance and branch/doc growth gaps are closed with deterministic budgets
Given R54 closure markers,
When tests validate governance-remediation budget, branch cleanup target math, and doc-contract non-regression caps,
Then all budget/status invariants pass fail-closed.

### AC-4: kamn-core module stagnation gap is closed with enforceable activation contract
Given R54 closure markers,
When module activation markers are validated,
Then snapshot module count and next-release activation minimum are present and internally consistent.

### AC-5: Recurring untracked `specs/` contamination is eliminated
Given the spec-volume non-regression lane,
When current spec-dir count is computed,
Then tracked-only semantics are used so untracked local directories do not trip the contract.

### AC-6: Regression lanes remain green after closure changes
Given the full change set,
When docs-contract and quality lanes run,
Then targeted test lanes, fmt, and clippy pass.

## Conformance Cases

| ID | AC | Tier | Case |
|---|---|---|---|
| C-01 | AC-1 | Functional | R54 closure marker family exists with `resolved_count == total_count == 6` and `all_resolved` status. |
| C-02 | AC-2 | Regression | R54 headings/markers comply with moratorium and marker-inflation closure semantics. |
| C-03 | AC-3 | Regression | Governance-remediation budget markers satisfy policy max; branch/doc budget formulas pass. |
| C-04 | AC-4 | Functional | kamn-core activation markers exist with next-release minimum >= 1 and valid status. |
| C-05 | AC-5 | Regression | Spec-volume contract counts tracked top-level spec dirs only and ignores untracked contamination. |
| C-06 | AC-6 | Integration | `review_r53_docs_contract` + `review_r50_spec_volume_remediation_docs_contract` + fmt/clippy pass. |

## Success Metrics / Observable Signals
- R54 unresolved-item statuses change from unresolved/worsened/recurring to explicit closure markers with fail-closed verification.
- Tracked-only spec-dir counting prevents local untracked `specs/` dirs from breaking non-regression gates.
- Required regression lanes pass without adding a new doc-contract test file.
