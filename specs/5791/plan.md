# Plan: Issue #5791 — Enforce R54+ Governance-Remediation Commit-Budget Docs-Contract

- Issue: #5791
- Spec: `specs/5791/spec.md`
- Status: Reviewed
- Last Updated: 2026-02-22

## Implementation Approach
1. Add RED test in `review_r53_docs_contract.rs` for R54+ governance-remediation budget markers and consistency.
2. Execute targeted test expecting failure due missing policy baseline.
3. Add policy file `docs/review/governance-remediation-budget.policy`.
4. Re-run targeted and integration contract lanes.
5. Update active milestone index with completed slice and preserve spec-cap by removing one archived pointer-only spec directory.

## Affected Modules
- `crates/kamn-core/tests/review_r53_docs_contract.rs`
- `docs/review/governance-remediation-budget.policy`
- `specs/5791/{spec.md,plan.md,tasks.md}`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks / Mitigations
- Risk: adding `specs/5791/` can exceed spec-dir cap.
  - Mitigation: remove one archived pointer-only spec directory in the same change set.
- Risk: division-by-zero semantics for item_count=0.
  - Mitigation: define deterministic 0.0 behavior and enforce status consistency accordingly.

## Interfaces / Contracts
- New policy baseline: `docs/review/governance-remediation-budget.policy`.
- R54+ review docs-contract budget marker family:
  - `r<release>_review_governance_remediation_budget_schema_version`
  - `r<release>_review_governance_remediation_item_count`
  - `r<release>_review_governance_remediation_commit_count`
  - `r<release>_review_governance_remediation_commits_per_item`
  - `r<release>_review_governance_remediation_budget_max_commits_per_item`
  - `r<release>_review_governance_remediation_budget_status`

## ADR
- None required.
