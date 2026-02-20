# Issue #5297 Plan

## Objective
Add a deterministic Phase-6 runtime evidence bundle projection that converts scheduler-cycle outcomes and runtime state into one canonical report payload.

## Approach
1. Add evidence reason markers and input/output contract types.
2. Implement evidence projection function with deterministic ordering and payload-shape validation:
   - applied cycles require execution + budget payload,
   - deferred cycles require no execution/budget payload.
3. Add conformance tests for applied/deferred projections and invalid payload fail-closed paths.
4. Add ops-doc markers and update milestone trackers.

## Affected Modules
- `crates/kamn-core/src/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ops/configuration.md`
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`

## Risks and Mitigations
- Risk: ambiguous evidence semantics between deferred and applied branches.
  - Mitigation: strict validation and fail-closed reason markers for branch/payload mismatches.
- Risk: non-deterministic artifact ordering in evidence output.
  - Mitigation: explicit deterministic sorting by partition month + partition name.

## Interfaces / Contracts
- New public evidence projection API over existing Phase-6 cycle/runtime contracts; no schema/wire changes.

## ADR
- Not required (no new dependency/protocol/schema change).
