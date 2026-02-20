# Issue #5267 Plan

- Issue: #5267
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Add a new `data_layer_m1_anchoring_orchestrator` module in `kamn-core`.
2. Implement orchestrator state that owns:
   - M1 scheduler policy,
   - M1 anchoring worker (client-backed).
3. Implement `tick` planning flow:
   - evaluate scheduler trigger,
   - assemble deterministic merkle batch from pending messages,
   - submit anchor through worker,
   - project persistence plan for adapter lifecycle methods.
4. Enforce fail-closed validation for:
   - invalid scheduling/submission timestamps,
   - missing/invalid confirmation metadata when anchor finality is `Final`.
5. Add targeted tests for defer/plan/reject/fail-closed branches and live adapter application.

## Affected Areas
- `crates/kamn-core/src/data_layer_m1_anchoring_orchestrator.rs` (new)
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m1_anchoring_orchestrator.rs` (new)
- `crates/kamn-core/tests/data_layer_postgres_execution_adapter.rs`
- `fixtures/ci/kamn_core_public_api_surface_baseline.env` (if required by policy)
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- `specs/5267/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: orchestration metadata diverges from adapter lifecycle API expectations.
  - Mitigation: integration test applies projected plan through adapter lifecycle methods.
- Risk: final receipt confirmation requirements are ambiguous.
  - Mitigation: explicit fail-closed error when final receipt arrives without valid confirmation metadata.
- Risk: API surface ratchet failures.
  - Mitigation: update baseline fixture only for net-new intentional exports.

## Interfaces / Contracts
- New orchestrator public types for pending inputs, persistence plan, tick outcome, and fail-closed errors.
- Existing adapter lifecycle methods remain unchanged and are consumed by integration tests.

## ADR
Not required; this is an incremental Phase-3 integration layer under existing architecture.
