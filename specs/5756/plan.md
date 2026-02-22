# Plan: #5756 Reconcile R52 Post-Publication Branch-Hygiene Status Markers

## Approach
1. Create lifecycle artifacts and add milestone slice 29 in-progress marker.
2. RED: extend `review_r52_branch_hygiene_reconciliation_docs_contract.rs` with new branch-status
   reconciliation assertions; run targeted test expecting failure.
3. Implement: add branch-status reconciliation marker contract text to `docs/review/README.md` and
   additive marker block in `docs/review/gaps-and-issues-r52.md`.
4. Implement: perform compensating single archived issue-spec pair cleanup for issue `3873` and
   update archive index.
5. GREEN: rerun targeted docs-contract tests + non-regression/archive-policy checks.
6. Verify formatting/lint and finish closure artifacts.

## Affected Modules / Files
- `docs/review/README.md`
- `docs/review/gaps-and-issues-r52.md`
- `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `specs/archive/index.md`
- `specs/5756/spec.md`
- `specs/5756/plan.md`
- `specs/5756/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: accidental rewrite of baseline branch-status rows.
  - Mitigation: additive marker section only + explicit baseline-line assertions.
- Risk: inconsistency with existing branch cleanup markers.
  - Mitigation: cross-marker consistency assertions in docs-contract tests.
- Risk: cap breach due new lifecycle spec directory.
  - Mitigation: compensating archive cleanup + archive-policy verification.

## Interfaces / Contracts
New marker keys in `docs/review/gaps-and-issues-r52.md`:
- `r52_review_post_publication_branch_hygiene_status_reconciliation_schema_version`
- `r52_review_branch_hygiene_snapshot_status`
- `r52_review_branch_hygiene_snapshot_branch_count`
- `r52_review_branch_hygiene_post_publication_pre_cleanup_count`
- `r52_review_branch_hygiene_post_publication_post_cleanup_count`
- `r52_review_branch_hygiene_post_publication_status`
- `r52_review_branch_hygiene_snapshot_rows_preserved`

## ADR
No ADR required (no architecture/protocol/dependency changes).
