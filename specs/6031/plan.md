# Plan: Issue #6031

## Approach
1. Add deterministic fixture helpers to register owner-scoped graph nodes and edges.
2. Write RED tests for owner-scope failure paths and trust propagation ranking/order invariants.
3. Keep production code unchanged unless tests surface a contract mismatch.
4. Run targeted M6 tests plus nearby data-layer slices for regression confidence.

## Affected Modules
- `crates/kamn-core/src/data_layer_m6_graph_integration.rs`

## Risks / Mitigations
- Risk: trust ranking assertions can become brittle if tied to incidental float precision.
  Mitigation: use deterministic weights and compare exact expected ordering/scores produced by current contract.
- Risk: edge registration failures may come from fixture mistakes rather than contract behavior.
  Mitigation: centralize fixture helpers and assert preconditions (`register_node`) explicitly.

## Interfaces / Contracts
- No public API changes.
- Test-only additions validating existing M6 graph integration contracts.
