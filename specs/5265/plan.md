# Issue #5265 Plan

- Issue: #5265
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Add a new M1 scheduler module that evaluates deterministic trigger conditions from:
   - pending message count,
   - oldest pending message age,
   - configured count/window thresholds.
2. Extend `DataLayerPgExecutionAdapter` with explicit merkle-batch persistence operations:
   - create batch row,
   - assign message rows to batch + leaf index,
   - mark batch submitted (tx hash + submitted timestamp),
   - mark batch confirmed (block height + confirmed timestamp).
3. Add fail-closed input validation at adapter boundary for invalid identifiers and transition payloads.
4. Add tests for scheduler determinism and live PostgreSQL lifecycle persistence transitions.
5. Run quality gates and update milestone/docs tracker references for `#5265`.

## Affected Areas
- `crates/kamn-core/src/data_layer_m1_batch_scheduler.rs` (new)
- `crates/kamn-core/src/data_layer_postgres_execution_adapter.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m1_batch_scheduler.rs` (new)
- `crates/kamn-core/tests/data_layer_postgres_execution_adapter.rs`
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- `specs/5265/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: SQL transition updates become non-deterministic across retries.
  - Mitigation: enforce deterministic state transition contract with explicit fail-closed checks and stable error variants.
- Risk: new adapter methods increase public API surface unexpectedly.
  - Mitigation: keep minimal method surface and update public API baseline only when required by policy gates.
- Risk: live DB integration tests may be env-dependent.
  - Mitigation: keep env-gated integration coverage and add deterministic unit/functional coverage for scheduler logic.

## Interfaces / Contracts
- New scheduler policy/decision structs with deterministic reason markers.
- New adapter merkle-batch persistence methods with explicit inputs and typed fail-closed errors.
- No new shell interfaces.

## ADR
Not required; this is an incremental implementation step for existing M1 story scope.
