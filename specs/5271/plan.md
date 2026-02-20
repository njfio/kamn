# Issue #5271 Plan

- Issue: #5271
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Extend `data_layer_m1_anchoring_orchestrator` with finality-observation and reconciliation projection types.
2. Add deterministic reconciliation function for planned outcomes:
   - validate commit-id consistency with submitted tx hash,
   - map finality (`Pending|Final|Failed`) to updated follow-up policy,
   - project confirmation metadata for `Final` when block height is provided.
3. Add fail-closed error variants and stable reason markers for mismatch/missing-block-height branches.
4. Add RED tests first for pending/final/failed mapping and fail-closed branches.
5. Add integration assertion path in adapter test to ensure reconciliation output is coherent with lifecycle persistence projection.

## Affected Areas
- `crates/kamn-core/src/data_layer_m1_anchoring_orchestrator.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m1_anchoring_orchestrator.rs`
- `crates/kamn-core/tests/data_layer_postgres_execution_adapter.rs`
- `fixtures/ci/kamn_core_public_api_surface_baseline.env` (if required by policy)
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- `specs/5271/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: reconciliation policy may drift from existing follow-up reason taxonomy.
  - Mitigation: explicit branch tests for pending/final/failed and mismatch paths.
- Risk: finality observation contract may introduce ambiguous block-height semantics.
  - Mitigation: fail-closed on `Final` without block height and document via stable reason marker.
- Risk: public API ratchet checks fail for new orchestrator exports.
  - Mitigation: update baseline fixture only for intentional net-new exports.

## Interfaces / Contracts
- Add finality observation input and reconciliation output types in orchestrator module.
- Keep existing adapter lifecycle methods unchanged.
- Keep previous submission-time follow-up policy projection intact.

## ADR
Not required; this is an incremental Phase-3 contract integration slice.
