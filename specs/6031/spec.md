# Spec: Issue #6031 - Add core invariants unit tests for data_layer_m6_graph_integration

- Issue: #6031
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #5976

## Problem Statement
`crates/kamn-core/src/data_layer_m6_graph_integration.rs` currently has only helper-function unit tests and lacks direct coverage for owner-scoped node/edge lifecycle, trust propagation ranking, and portable projection contracts.

## Scope
In scope:
- Add direct `#[cfg(test)]` coverage for `DataLayerM6GraphRegistry` contracts.
- Validate deterministic node/edge sequencing and portable projection ordering.
- Validate fail-closed owner-scope isolation and duplicate edge id rejection.
- Validate deterministic trust propagation ranking behavior and reason-code projection.

Out of scope:
- Changes to M6 production behavior or query algorithm.
- Cross-service/network integration wiring.
- M7+ module coverage.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: Node/edge registration persists deterministic sequence ordering and portable export projection ordering.
- AC-2: Cross-owner edge references and duplicate edge IDs fail closed with stable error taxonomy.
- AC-3: Trust propagation query yields deterministic ranked results with stable reason code and bounded row count.

## Conformance Cases
- C-01 (Unit, AC-1): Register owner-scoped nodes/edges and verify sequence counters plus sorted portable edge projection by `edge_id`.
- C-02 (Regression, AC-2): Registering an edge that references a node outside owner scope yields `OwnerScopeViolation { reason_code: m6_graph_cross_owner_edge_denied }`.
- C-03 (Functional, AC-2): Reusing an existing edge_id fails with `DuplicateEdgeId`.
- C-04 (Conformance, AC-3): Trust propagation from source agent returns deterministic ranking order, hop counts, reason code `m6_graph_trust_score_ranked`, and honors `limit`.

## Success Metrics / Observable Signals
- New direct M6 graph registry tests pass in `kamn-core`.
- AC-to-test mapping is explicit in PR verification table.
- M6 no longer appears in zero-core-contract-coverage tracking.
