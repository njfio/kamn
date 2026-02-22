# Plan: #5768 Add R52 Feat-Labeling Post-Publication Reconciliation Contract

## Approach
1. Add lifecycle artifacts and mark milestone slice 33 in progress.
2. RED: extend `review_r52_branch_hygiene_reconciliation_docs_contract.rs` with feat-labeling
   reconciliation assertions and run targeted lane expecting failure.
3. Implement: add README contract section and additive marker block to
   `docs/review/gaps-and-issues-r52.md`.
4. Implement: perform compensating archive cleanup for one archived issue-spec pair and update
   `specs/archive/index.md`.
5. GREEN: rerun docs-contract target lane and spec-volume/archive-policy checks.
6. Verify formatting and lint gates.

## Affected Modules / Files
- `docs/review/README.md`
- `docs/review/gaps-and-issues-r52.md`
- `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`
- `specs/archive/index.md`
- `specs/5768/spec.md`
- `specs/5768/plan.md`
- `specs/5768/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: ratio marker drift from snapshot counts.
  - Mitigation: strict numeric consistency assertion (`abs(delta) <= 0.0001`).
- Risk: baseline priority row accidental edit.
  - Mitigation: additive-only marker section + explicit row-preservation assertion.
- Risk: `specs/` cap breach.
  - Mitigation: compensating archive cleanup + archive-policy checker.

## Interfaces / Contracts
New marker keys in `docs/review/gaps-and-issues-r52.md`:
- `r52_review_post_publication_feat_labeling_reconciliation_schema_version`
- `r52_review_feat_labeling_snapshot_mislabeled_feat_count`
- `r52_review_feat_labeling_snapshot_total_feat_count`
- `r52_review_feat_labeling_snapshot_mislabeled_ratio`
- `r52_review_feat_labeling_recommended_prefixes_csv`
- `r52_review_feat_labeling_post_publication_status`
- `r52_review_feat_labeling_snapshot_rows_preserved`

## ADR
No ADR required (no architecture/protocol/dependency changes).
