# Issue #5263 Plan

- Issue: #5263
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Implement a new Phase-2 operational pipeline module in `kamn-core` that:
   - encrypts canonical envelope payload via `DirectMessageCryptoEngine`,
   - derives `DataLayerM0EnvelopeRecord`,
   - computes deterministic M3 blind-index tokens.
2. Add deterministic input/output contracts and error taxonomy.
3. Extend PostgreSQL execution adapter insert path to persist provided blind-index map as deterministic JSON.
4. Add tests for deterministic pipeline output, invalid-input failure paths, and adapter insert/search integration.
5. Update milestone/docs/spec artifacts and run verification gates.

## Affected Areas
- `crates/kamn-core/src/data_layer_phase2_crypto_blind_index_pipeline.rs` (new)
- `crates/kamn-core/src/data_layer_postgres_execution_adapter.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_phase2_crypto_blind_index_pipeline.rs` (new)
- `crates/kamn-core/tests/data_layer_postgres_execution_adapter.rs`
- `fixtures/ci/kamn_core_public_api_surface_baseline.env` (if public surface threshold requires update)
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- `specs/5263/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: Pipeline introduces public API growth beyond public-surface fail threshold.
  - Mitigation: keep API narrow and use internal helpers; adjust export strategy conservatively.
- Risk: JSON serialization of blind-index map can drift.
  - Mitigation: deterministic BTreeMap iteration + explicit escaping helper and regression assertions.
- Risk: Live DB tests become flaky.
  - Mitigation: keep DB-dependent test env-gated and retain deterministic unit coverage for core logic.

## Interfaces / Contracts
- New pipeline request/output structs with deterministic ordering semantics.
- Adapter insert method extended to accept caller-provided blind-index map without panics.
- Existing insert method remains as compatibility wrapper using empty map.

## ADR
Not required; this is an incremental runtime implementation of established Phase-2 plan.
