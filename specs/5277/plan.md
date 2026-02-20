# Issue #5277 Plan

- Issue: #5277
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Extend `data_layer_postgres_repository_bridge` with Timescale projection contracts for M7 ingest and owner rollup query paths.
2. Add Timescale capability config plus fail-closed reason-coded error variants for extension-unavailable and invalid bucket-window inputs.
3. Add RED tests in `data_layer_postgres_repository_bridge` for deterministic descriptor projection and fail-closed branches.
4. Export new Timescale bridge APIs through `lib.rs`.
5. Run scoped verification (`fmt`, strict `clippy`, targeted bridge tests, public API surface policy).

## Affected Areas
- `crates/kamn-core/src/data_layer_postgres_repository_bridge.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_postgres_repository_bridge.rs`
- `fixtures/ci/kamn_core_public_api_surface_baseline.env` (if intentional API delta requires baseline update)
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- `specs/5277/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: bucket-window semantics drift from M7 aggregate windows.
  - Mitigation: validate bucket-window against explicit allowed windows and pin with regression tests.
- Risk: Timescale contract might overfit one rollup window.
  - Mitigation: support deterministic hourly/daily window markers and fail closed otherwise.
- Risk: public API ratchet may fail due new exports.
  - Mitigation: update baseline fixture only when delta is intentional and validated.

## Interfaces / Contracts
- New deterministic Timescale projection APIs and request/config structs in the PG repository bridge.
- No live SQL execution path changes in this issue.
- M7 contract record/query types remain source-of-truth bridge inputs.

## ADR
Not required; this is an incremental bridge-contract integration slice under existing Phase-4 architecture.
