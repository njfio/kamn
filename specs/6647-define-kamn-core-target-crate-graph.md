# 6647 Define KAMN Core Target Crate Graph

## Objective

Define the target crate graph that should exist after `kamn-core` decomposition, including explicit allowed dependency directions, a concrete inversion plan for `kamn-types`, and candidate module-to-crate mapping for the first domain extraction waves.

## Inputs/Outputs

- Inputs:
  - Current workspace crate graph from `cargo metadata`
  - Existing `docs/architecture/kamn-core-module-map.md`
  - Existing `docs/architecture/kamn-types.md`
  - Current `kamn-types` dependency on `kamn-core`
- Outputs:
  - New architecture document describing the target crate graph and migration rules
  - Updated architecture index and `kamn-types` boundary doc reflecting the inversion plan
  - Contract tests pinning the new architecture markers and index links

## Boundaries/Non-goals

- Do not move code between crates in this issue
- Do not rename crates or remove compatibility shims in this issue
- Do not change runtime behavior or public APIs in this issue
- Do not redefine the existing decomposition tranche map except where it is referenced by the new target graph

## Failure Modes

- The architecture document omits allowed dependency directions, so follow-up extraction issues can reintroduce coupling
- The document does not explain how `kamn-types` becomes foundational, leaving the inversion unresolved
- Candidate module groupings are too vague to guide extraction issues
- The architecture index and crate docs drift from the new target graph contract
- No test pins the architecture markers, allowing silent design drift

## Acceptance Criteria

- [ ] A new architecture document defines the post-`kamn-core` target crate graph
- [ ] The document names proposed domain slices, including governance, escrow/task settlement, and compliance/content policy
- [ ] Allowed dependency directions and temporary bridge rules are explicit
- [ ] The target state makes `kamn-types` foundational with no dependency on `kamn-core`
- [ ] Candidate modules from current `kamn-core` are mapped into future crates or retained-core domains
- [ ] Migration order is documented
- [ ] At least one contract test pins the new graph markers and architecture index link

## Files To Touch

- `specs/6647-define-kamn-core-target-crate-graph.md`
- `docs/architecture/kamn-core-target-crate-graph.md`
- `docs/architecture/README.md`
- `docs/architecture/kamn-types.md`
- `crates/kamn-core/tests/kamn_core_target_crate_graph_docs.rs`

## Error Semantics

- Missing architecture markers or index links must fail deterministically in contract tests
- The architecture document must fail closed on layering by stating forbidden dependency directions explicitly
- Temporary bridge rules must be explicit so follow-up extraction work cannot assume silent compatibility behavior

## Test Plan

- Run `cargo test -p kamn-core --test kamn_core_target_crate_graph_docs -- --nocapture`
- Run `cargo test -p kamn-types --test identity_boundary_contract -- --nocapture`
- Run `cargo test -p kamn-core --test kamn_core_decomposition_map_docs -- --nocapture`
- Verify the architecture index links the new target graph document
