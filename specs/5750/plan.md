# Plan: #5750 Reconcile R52 Post-Publication Spec-Volume Guardrail Status Markers

## Approach
1. Add lifecycle artifacts and milestone slice marker for #5750.
2. RED: extend `review_r50_spec_volume_remediation_docs_contract.rs` with expected
   post-publication guardrail reconciliation markers and consistency assertions; run the targeted test
   to confirm failure before docs updates.
3. Implement: add additive reconciliation section and marker set in
   `docs/review/gaps-and-issues-r52.md` while preserving historical snapshot lines.
4. Implement: perform compensating single archived issue-spec pair cleanup (`specs/<id>/ARCHIVED.md`,
   `specs/archive/<id>/`, `specs/archive/index.md`) to keep top-level `specs/` directory count within
   the non-regression cap after adding `specs/5750`.
5. GREEN: rerun targeted docs-contract test and companion regression suites.
6. Verify formatting/lint and close issue via closure artifact update.

## Affected Modules / Files
- `docs/review/gaps-and-issues-r52.md`
- `docs/review/README.md`
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `specs/archive/index.md`
- `specs/5750/spec.md`
- `specs/5750/plan.md`
- `specs/5750/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: unintentionally mutating historical snapshot values.
  - Mitigation: additive section only; explicit baseline-preservation assertion in test.
- Risk: marker precision drift for ratio values.
  - Mitigation: assert non-regression bounds with tolerant float checks and direct target comparison.
- Risk: non-regression cap violation due mandatory `specs/5750` lifecycle directory.
  - Mitigation: compensating single archive cleanup + archive-policy checker verification.

## Interfaces / Contracts
- Review marker contract keys in `docs/review/gaps-and-issues-r52.md`:
  - `r52_review_post_publication_spec_volume_guardrail_reconciliation_schema_version`
  - `r52_review_spec_volume_guardrail_snapshot_spec_dir_count`
  - `r52_review_spec_volume_guardrail_snapshot_module_count`
  - `r52_review_spec_volume_guardrail_post_publication_spec_dir_count`
  - `r52_review_spec_volume_guardrail_post_publication_module_count`
  - `r52_review_spec_volume_guardrail_post_publication_ratio`
  - `r52_review_spec_volume_guardrail_target_ratio_max`
  - `r52_review_spec_volume_guardrail_post_publication_status`

## ADR
No ADR required (no architecture/dependency/protocol changes).
