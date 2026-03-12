# 6931-split-data-layer-m6-graph-integration

## Objective
Split `crates/kamn-core/src/data_layer_m6_graph_integration.rs` into bounded, concern-based modules while preserving owner-scoped graph registration, trust propagation ranking, portability projection, and existing deterministic error semantics.

## Inputs/Outputs
- Inputs:
  - graph node and edge registration payloads
  - trust propagation queries
  - owner DID scope and node identifiers
  - edge portability export requests
- Outputs:
  - unchanged M6 graph registration and trust-propagation behavior
  - a thin root shell in `data_layer_m6_graph_integration.rs`
  - bounded sibling modules for models, registry, query/planning, portability/export, support/error handling, and tests
  - a hard-fail extraction contract for module layout and root budget

## Boundaries/Non-goals
- No changes to public reason codes, engine markers, or portability profile markers
- No changes to trust score semantics or traversal rules except extraction-safe internal refactors
- No new dependencies
- No unrelated data-layer refactors outside the M6 graph integration surface

## Failure modes
- invalid owner DIDs remain fail-closed
- empty node/edge identifiers remain fail-closed
- duplicate node or edge identifiers remain fail-closed
- cross-owner and missing-node edge registration remains fail-closed
- invalid trust propagation query parameters remain fail-closed
- extraction contract fails if root shell or module layout regress

## Acceptance criteria
- [ ] `crates/kamn-core/src/data_layer_m6_graph_integration.rs` becomes a thin root shell under the active file-size budget
- [ ] graph integration concerns are split into bounded modules with clear responsibilities
- [ ] a hard-fail extraction contract enforces the root shell and module layout
- [ ] existing M6 graph integration tests remain green without semantic drift
- [ ] touched-Rust size policy returns `policy_decision=GO`
- [ ] final spec records test evidence and any deviations

## Files to touch
- `crates/kamn-core/src/data_layer_m6_graph_integration.rs`
- `crates/kamn-core/src/data_layer_m6_graph_integration/`
- `crates/kamn-core/tests/data_layer_m6_graph_integration_module_extraction_contract.rs`
- `specs/6931-split-data-layer-m6-graph-integration.md`

## Error semantics
- Preserve existing typed `DataLayerM6GraphIntegrationError` behavior and all stable reason markers
- Preserve hard-fail behavior for invalid input and authorization/scope denials
- Do not introduce silent fallback or relaxed graph registration/query behavior

## Test plan
- Add a red extraction contract that fails while `data_layer_m6_graph_integration.rs` is still monolithic
- Run the extraction contract green once the split is in place
- Run the real `data_layer_m6_graph_integration` test target after extraction
- Run touched-Rust size policy against the staged write set
