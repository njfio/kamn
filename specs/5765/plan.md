# Plan: #5765 Add R52 Governance-Feature 70/30 Target Reconciliation Contract

## Approach
1. Add lifecycle artifacts and mark milestone slice 32 in progress.
2. RED: extend `review_r52_branch_hygiene_reconciliation_docs_contract.rs` with new governance-
   feature target reconciliation assertions and run the targeted lane expecting failure.
3. Implement: add contract section to `docs/review/README.md` and additive marker block in
   `docs/review/gaps-and-issues-r52.md`.
4. Implement: perform compensating archive cleanup for one archived issue-spec pair and update
   `specs/archive/index.md`.
5. GREEN: rerun targeted docs-contract lane + spec-volume/archive-policy checks.
6. Verify formatting and lint gates.

## Affected Modules / Files
- `docs/review/README.md`
- `docs/review/gaps-and-issues-r52.md`
- `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`
- `specs/archive/index.md`
- `specs/5765/spec.md`
- `specs/5765/plan.md`
- `specs/5765/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: mismatch between snapshot markers and new reconciliation markers.
  - Mitigation: assert strict equality against Section 5.3 ratio markers.
- Risk: accidental baseline narrative edits.
  - Mitigation: additive section only + explicit preservation assertions in tests.
- Risk: `specs/` cap breach.
  - Mitigation: compensating archived pair cleanup + archive-policy checker.

## Interfaces / Contracts
New marker keys in `docs/review/gaps-and-issues-r52.md`:
- `r52_review_post_publication_governance_feature_target_reconciliation_schema_version`
- `r52_review_governance_feature_snapshot_governance_ratio`
- `r52_review_governance_feature_snapshot_feature_ratio`
- `r52_review_governance_feature_target_governance_ratio_max`
- `r52_review_governance_feature_target_feature_ratio_min`
- `r52_review_governance_feature_target_status`
- `r52_review_governance_feature_snapshot_rows_preserved`

## ADR
No ADR required (no architecture/protocol/dependency changes).
