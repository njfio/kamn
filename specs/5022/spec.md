# Issue #5022 Spec

- Title: Task: M6 deliver Apache AGE graph schema and trust propagation query service
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD M6 requires a knowledge-graph contract surface for owner-scoped graph nodes/edges,
bounded trust propagation scoring, and portability boundaries so graph data can move
between Apache AGE and fallback graph engines without semantic drift.
Current codebase has trust and reputation modules, but no dedicated M6 graph contract
module that binds schema registration, propagation query semantics, and export portability.

PRD mapping:
- Section 7 (Knowledge Graph)
- Section 7.1 (graph schema / node and edge model)
- Section 7.2 (query patterns and trust propagation)
- Section 7.3 (event-driven graph updates and trust score refresh)
- Milestone table M6 deliverables (AGE schema + trust propagation + query API)

## Acceptance Criteria
- AC-1: Graph registry contract supports deterministic owner-scoped node and edge
  registration for M6 entities (agents, owners, escrows, capabilities) with fail-closed validation.
- AC-2: Trust propagation query computes bounded-depth owner-scoped trust scores with deterministic ordering and stable reason markers.
- AC-3: Portability contract exports deterministic edge projections suitable for AGE/openCypher and fallback graph adapters.
- AC-4: Cross-owner graph access is denied fail-closed for registration and trust-propagation queries.
- AC-5: Shell/workflow/python/template LOC remains unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust M6 module in `kamn-core` for graph node/edge registration, trust propagation,
  and portability projection contracts.
- Conformance tests for deterministic propagation ranking, owner isolation, and portable edge projection outputs.
- Public API exports for downstream M7+ integration lanes.

Out of scope:
- Live PostgreSQL/AGE extension DDL or runtime SQL execution.
- Background trigger schedulers and batch reconciliation jobs.
- New dependencies, protocol/wire-format changes, or shell/python workflow additions.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Register owner graph nodes/edges with stable ids and relationship labels | Records are accepted deterministically and registry state is append-only |
| C-02 | AC-1/AC-4 | Unit | Attempt invalid DID scope or cross-owner edge registration | Fail-closed typed errors are returned |
| C-03 | AC-2 | Conformance | Run trust propagation from a source agent with bounded depth | Deterministic ranking/scores are returned with stable reason markers |
| C-04 | AC-3 | Regression | Export owner graph edge projection to portability interface | Deterministic projection ordering and field completeness are preserved |
| C-05 | AC-4 | Conformance | Query trust propagation for a requester outside owner scope | Access denied with stable reason marker |
| C-06 | AC-5 | Regression | Inspect issue diff for shell/python/workflow/template files | Net shell-surface delta remains zero |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m6_graph_integration`
- `cargo test -p kamn-core spec_c0`
- `cargo test -p kamn-core`
- Shell governance scripts are not required because shell/workflow surfaces are unchanged.

## Success Metrics
- All ACs map to passing `spec_c0x_*` conformance tests.
- M6 contracts are exported via `kamn_core` for downstream integration lanes.
- Shell-to-Rust ratio direction is improved/neutral through Rust-only changes.

## Verification Evidence
- RED: `cargo test -p kamn-core --test data_layer_m6_graph_integration` failed before implementation with unresolved `DataLayerM6*` symbols.
- GREEN: `cargo test -p kamn-core --test data_layer_m6_graph_integration` passed after module implementation and exports.
- REGRESSION: `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, and `cargo test -p kamn-core` pass.

## AC Verification
| AC | Result | Tests |
|---|---|---|
| AC-1 | ✅ | `spec_c01_graph_registry_accepts_owner_scoped_node_and_edge_contracts`; `spec_c02_cross_owner_graph_edge_registration_is_denied_fail_closed` |
| AC-2 | ✅ | `spec_c03_trust_propagation_returns_deterministic_ranked_results` |
| AC-3 | ✅ | `spec_c04_portability_projection_is_deterministic_and_complete` |
| AC-4 | ✅ | `spec_c02_cross_owner_graph_edge_registration_is_denied_fail_closed`; `spec_c05_trust_propagation_denies_requester_outside_owner_scope` |
| AC-5 | ✅ | Diff inspection for issue files confirms Rust-only surface |

## Shell Surface Markers
- shell_loc_delta_actual: 0
- rust_loc_delta_actual: +846
- shell_to_rust_ratio_delta_actual: -0.006010
- shell_surface_ratio_target_status: improved
