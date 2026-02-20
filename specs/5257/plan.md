# Issue #5257 Plan

- Issue: #5257
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Add a new contract module `data_layer_postgres_repository_bridge` in `kamn-core`.
2. Define deterministic descriptor structs for:
   - message insert,
   - message lookup by id,
   - owner-scoped blind-index search.
3. Define typed requester session projection from DID input and bind it to each descriptor.
4. Add deterministic projection for M2 default RLS policies into SQL statements.
5. Add integration tests first (RED), then implement minimum code to satisfy ACs.

## Affected Areas
- `crates/kamn-core/src/data_layer_postgres_repository_bridge.rs` (new)
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_postgres_repository_bridge.rs` (new)
- `specs/5257/{spec.md,plan.md,tasks.md}`
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`

## Risks and Mitigations
- Risk: SQL descriptor strings drift in format and break downstream consumers.
  - Mitigation: freeze expected SQL in tests and require deterministic order/bind markers.
- Risk: DID boundary remains stringly-typed and allows invalid session metadata.
  - Mitigation: parse requester DID into typed identity for session projection.
- Risk: Over-coupling to current migration column naming.
  - Mitigation: keep descriptors focused on stable table/field markers already codified in `#5255` migration contract tests.

## Interfaces / Contracts
- Bridge APIs return immutable descriptor types and never execute SQL.
- RLS projection APIs consume `data_layer_m2_default_rls_policies()` and emit deterministic SQL statements.
- Failures return structured bridge errors (`field`, `reason_code`, `detail` where relevant).

## ADR
Not required (contract-layer bridge scaffolding; no dependency/protocol decision yet).
