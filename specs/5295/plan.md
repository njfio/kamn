# Issue #5295 Plan

## Objective
Add a stateful Phase-6 scheduler runtime contract that tracks checkpoint/counter state across cycles while composing existing scheduler-cycle trigger and guarded execution behavior.

## Approach
1. Add runtime state and runtime wrapper types for Phase-6 scheduler execution.
2. Add deterministic reason markers for runtime initialization/applied/deferred/fail-closed signals.
3. Implement runtime `run_cycle` method that:
   - enforces monotonic `now_epoch_seconds`,
   - calls `data_layer_m10_execute_phase6_scheduler_cycle`,
   - updates state counters/checkpoints deterministically.
4. Add conformance tests for initialization, deferred/apply transitions, preflight fail-closed, and clock regression fail-closed.
5. Update docs and tracker files.

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
- Risk: state counters drift from actual cycle outcomes.
  - Mitigation: explicit per-outcome update branches with tests for each transition.
- Risk: clock regression accepted silently.
  - Mitigation: fail-closed monotonic time validation with stable scheduler-signal reason marker.

## Interfaces / Contracts
- New public stateful runtime API over existing M8/M10 scheduler-cycle contracts; no schema/wire changes.

## ADR
- Not required (no new dependency/protocol/schema change).
