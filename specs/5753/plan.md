# Plan: #5753 Reconcile R52 Post-Publication Priority Summary Status Markers

## Approach
1. Create lifecycle artifacts and add milestone slice 28 in-progress marker.
2. RED: extend `review_r52_branch_hygiene_reconciliation_docs_contract.rs` to require new
   priority-summary reconciliation markers and consistency checks; run targeted test expecting failure.
3. Implement: add marker contract section to `docs/review/README.md` and additive marker block to
   `docs/review/gaps-and-issues-r52.md` while preserving baseline Priority Summary rows.
4. Implement: perform compensating single archive cleanup for issue `3872` and update
   `specs/archive/index.md`.
5. GREEN: rerun targeted docs-contract tests and archive-policy/non-regression checks.
6. Verify formatting/lint and complete closure artifacts.

## Affected Modules / Files
- `docs/review/README.md`
- `docs/review/gaps-and-issues-r52.md`
- `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `specs/archive/index.md`
- `specs/5753/spec.md`
- `specs/5753/plan.md`
- `specs/5753/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: accidental rewrite of baseline priority rows.
  - Mitigation: additive section only + explicit baseline-row assertion in docs-contract test.
- Risk: marker drift across sections.
  - Mitigation: cross-section consistency assertions in targeted test.
- Risk: non-regression cap breach due new `specs/5753` directory.
  - Mitigation: compensating archive cleanup + archive-policy checker.

## Interfaces / Contracts
New marker keys in `docs/review/gaps-and-issues-r52.md`:
- `r52_review_post_publication_priority_reconciliation_schema_version`
- `r52_review_priority_critical_cli_compile_status_post_publication`
- `r52_review_priority_medium_activity_ratio_marker_status_post_publication`
- `r52_review_priority_high_spec_volume_guardrail_status_post_publication`
- `r52_review_priority_summary_snapshot_preserved`

## ADR
No ADR required (no architecture/dependency/protocol changes).
