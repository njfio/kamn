# Issue #5273 Plan

- Issue: #5273
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Extend `data_layer_postgres_repository_bridge` with pgvector projection contracts:
   - new operation kinds for M5 pgvector insert/search,
   - config + request structures for deterministic projection inputs,
   - stable reason markers and fail-closed error variants for extension unavailability and dimension mismatch.
2. Keep projection deterministic and side-effect free (no SQL execution).
3. Add RED tests in `data_layer_postgres_repository_bridge` covering deterministic insert/search descriptors and fail-closed branches.
4. Export new projection APIs from `lib.rs` for downstream adapter composition.
5. Run scoped verification (`fmt`, strict `clippy`, targeted tests).

## Affected Areas
- `crates/kamn-core/src/data_layer_postgres_repository_bridge.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_postgres_repository_bridge.rs`
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- `specs/5273/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: projection contract may diverge from M5 privacy-mode assumptions.
  - Mitigation: build test fixtures from real M5 registry append outputs.
- Risk: dimension mismatch semantics could become ambiguous across insert and query flows.
  - Mitigation: use one stable reason code and explicit expected/found fields for both paths.
- Risk: public API ratchet may fail due new exports.
  - Mitigation: update public API baseline only if policy fails and deltas are intentional.

## Interfaces / Contracts
- New deterministic pgvector projection APIs in repository bridge.
- No changes to live SQL execution paths in this issue.
- Existing M5 contract types remain source-of-truth for embedding/query inputs.

## ADR
Not required; this is an incremental bridge-contract integration slice under existing Phase-4 architecture.
