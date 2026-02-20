# Issue #5291 Plan

## Objective
Add deterministic Phase-6 execution-tick budget guardrail contracts that classify orchestration report workload and fail closed on invalid limits.

## Approach
1. Extend M10 Phase-6 types with budget contract structs/enums:
   - budget limits
   - budget decision report
   - stable reason-marker constants
2. Implement budget evaluation function that consumes `DataLayerM10Phase6ExecutionTickReport` and validates limits.
3. Enforce deterministic exceeded-priority ordering across dimensions:
   - due candidates
   - shredded message count
   - projection count
   - archived entries count
4. Add conformance tests and docs marker assertions; update trackers.

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
- Risk: ambiguous over-budget reporting when multiple dimensions exceed limits.
  - Mitigation: explicit deterministic priority ordering and tests for it.
- Risk: invalid budget limits silently accepted.
  - Mitigation: fail-closed validation with stable invalid-budget marker.

## Interfaces / Contracts
- New public budget evaluator API over existing orchestration report type; no wire/schema changes.

## ADR
- Not required (no dependency/protocol/schema changes).
