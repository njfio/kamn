# Issue #5293 Plan

## Objective
Implement deterministic Phase-6 scheduler-cycle contracts that decide trigger/defer, enforce preflight budget admission, and execute retention+archival orchestration only when admitted.

## Approach
1. Add scheduler trigger policy/signal/decision types with stable reason markers.
2. Add scheduler-cycle request/report contracts that compose:
   - scheduler trigger evaluation,
   - preflight budget admission projection,
   - orchestration execution,
   - post-execution budget evidence.
3. Implement fail-closed preflight budget admission checks before execution mutations.
4. Add conformance tests for deferred/triggered paths and preflight overflow fail-closed behavior.
5. Update ops docs and tracker docs.

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
- Risk: scheduler cycle mutates state before budget checks.
  - Mitigation: explicit preflight admission check using deterministic workload estimates; fail closed before execution.
- Risk: ambiguous trigger reason ordering.
  - Mitigation: fixed precedence (`due threshold` then `interval` then `deferred`) with dedicated tests.

## Interfaces / Contracts
- New public scheduler trigger evaluator and scheduler-cycle executor APIs over existing M8/M10 registries.
- No wire-format or schema changes.

## ADR
- Not required (no new dependency/protocol/schema change).
