# Issue #5269 Plan

- Issue: #5269
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Extend `data_layer_m1_anchoring_orchestrator` with follow-up policy types and stable reason markers.
2. Add deterministic follow-up projection logic from `DataLayerM1AnchorResult`:
   - `NewSubmission` -> confirmation polling policy,
   - `RetryableInFlight` -> retry policy + backoff projection,
   - `FinalizedNoRetry` / `ConflictNoRetry` -> no-retry policy.
3. Embed follow-up policy metadata in planned/rejected outcomes where applicable.
4. Add targeted tests for:
   - deterministic follow-up projection mapping,
   - retryable duplicate pending receipt path,
   - conflict/no-retry path,
   - integration coherence with adapter lifecycle application tests.

## Affected Areas
- `crates/kamn-core/src/data_layer_m1_anchoring_orchestrator.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m1_anchoring_orchestrator.rs`
- `crates/kamn-core/tests/data_layer_postgres_execution_adapter.rs`
- `fixtures/ci/kamn_core_public_api_surface_baseline.env` (if required by policy)
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- `specs/5269/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: follow-up policy mapping drifts from existing retry-class semantics.
  - Mitigation: explicit mapping tests for every retry class and receipt finality branch.
- Risk: outcome type growth introduces public API surface drift errors.
  - Mitigation: update public API baseline fixture only for intended net-new exports.
- Risk: adapter integration tests become brittle if follow-up data is not wired consistently.
  - Mitigation: integration test asserts follow-up policy coherence alongside persistence-plan lifecycle calls.

## Interfaces / Contracts
- Add new public follow-up policy structs/enums in orchestrator module.
- Keep existing adapter persistence lifecycle methods unchanged.
- Keep orchestrator fail-closed confirmation metadata validation intact.

## ADR
Not required; this is an incremental Phase-3 runtime contract integration task.
