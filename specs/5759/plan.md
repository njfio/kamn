# Plan: #5759 Reconcile R52 Post-Publication Code-Quality Status Markers

## Approach
1. Create lifecycle artifacts and add milestone slice 30 in-progress marker.
2. RED: extend `review_r52_branch_hygiene_reconciliation_docs_contract.rs` to require new
   Section 4.3 code-quality reconciliation markers; run targeted test expecting failure.
3. Implement: add marker-contract guidance to `docs/review/README.md` and additive Section 4.3
   marker block to `docs/review/gaps-and-issues-r52.md`.
4. Implement: perform compensating archive cleanup for issue `3874` and update
   `specs/archive/index.md`.
5. GREEN: rerun targeted docs-contract tests + non-regression/archive-policy checks.
6. Verify formatting/lint and complete closure artifacts.

## Affected Modules / Files
- `docs/review/README.md`
- `docs/review/gaps-and-issues-r52.md`
- `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `specs/archive/index.md`
- `specs/5759/spec.md`
- `specs/5759/plan.md`
- `specs/5759/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: accidental edits to baseline Section 4.2 text.
  - Mitigation: additive Section 4.3 only + explicit baseline assertions in tests.
- Risk: inconsistency with existing quality-gate markers.
  - Mitigation: cross-marker consistency checks in docs-contract tests.
- Risk: non-regression cap breach from new lifecycle spec directory.
  - Mitigation: compensating archive cleanup + archive-policy checker.

## Interfaces / Contracts
New marker keys in `docs/review/gaps-and-issues-r52.md`:
- `r52_review_post_publication_code_quality_status_reconciliation_schema_version`
- `r52_review_code_quality_snapshot_status`
- `r52_review_code_quality_post_publication_workspace_gate_status`
- `r52_review_code_quality_post_publication_cli_compile_status`
- `r52_review_code_quality_snapshot_rows_preserved`

## ADR
No ADR required (no architecture/protocol/dependency changes).
