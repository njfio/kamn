# Issue #5275 Plan

- Issue: #5275
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Extend `data_layer_postgres_repository_bridge` with AGE projection contracts for M6 write/read pathways.
2. Add explicit AGE capability config, relation-kind validation, and fail-closed reason-coded error variants.
3. Add RED tests in `data_layer_postgres_repository_bridge` for deterministic descriptor projection and fail-closed branches.
4. Export new AGE bridge APIs through `lib.rs`.
5. Run scoped verification (`fmt`, strict `clippy`, targeted bridge tests).

## Affected Areas
- `crates/kamn-core/src/data_layer_postgres_repository_bridge.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_postgres_repository_bridge.rs`
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- `specs/5275/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: relation-kind mapping drifts from M6 contract semantics.
  - Mitigation: use M6-derived fixtures and explicit relation-kind validation tests.
- Risk: read-query projection may overfit one traversal pattern.
  - Mitigation: keep descriptor contract generic and deterministic, limited to owner-scoped trust propagation.
- Risk: public API ratchet can fail due new exports.
  - Mitigation: baseline update only if policy test requires and delta is intentional.

## Interfaces / Contracts
- New deterministic AGE projection functions and input structs in the PG repository bridge.
- No live SQL execution changes in this issue.
- M6 graph types remain source-of-truth inputs.

## ADR
Not required; this is an incremental bridge-contract slice under existing Phase-4 architecture.
