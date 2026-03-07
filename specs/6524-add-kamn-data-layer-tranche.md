## Objective

Add a dedicated `kamn-data-layer` extraction tranche to the
`kamn-core` decomposition roadmap so the remaining `data_layer_*` surface in
`kamn-core` is explicitly accounted for in the monolith-reduction plan.

## Inputs/Outputs

- Input:
  - `docs/architecture/kamn-core-module-map.md`
  - `crates/kamn-core/tests/kamn_core_decomposition_map_docs.rs`
  - existing partial extraction context from `crates/kamn-data-layer`
- Output:
  - updated decomposition markers and tranche table including
    `crates/kamn-data-layer`
  - updated docs contract coverage enforcing the new tranche shape

## Boundaries/Non-goals

- No production module moves or re-export changes
- No new crate creation
- No CI/workflow modifications
- No broader rewrite of unrelated architecture sections outside the missing
  decomposition tranche and its contract markers

## Failure modes

- The roadmap continues to omit the existing `kamn-data-layer` extraction seam
- Tranche-count and target-crate markers drift from the actual table contents
- The new tranche is too vague to identify the remaining `data_layer_*`
  ownership boundary in `kamn-core`

## Acceptance criteria

- [ ] The decomposition roadmap includes a dedicated tranche for the remaining
      `data_layer_*` extraction boundary
- [ ] The tranche names the module-group boundary, destination crate, and
      ordering rationale
- [ ] The tranche markers and table-shape docs test are updated and pass
- [ ] The spec records that this is planning-only with no production extraction
      change in this issue

## Files to touch

- `specs/6524-add-kamn-data-layer-tranche.md`
- `docs/architecture/kamn-core-module-map.md`
- `crates/kamn-core/tests/kamn_core_decomposition_map_docs.rs`

## Error semantics

- No runtime error behavior changes are allowed
- Docs contract tests must fail loudly if the roadmap markers and tranche table
  drift out of sync

## Test plan

1. Add a red-phase docs contract expectation for the new tranche count/target
   crate so the current roadmap fails.
2. Update the roadmap markers and tranche table with the `kamn-data-layer`
   tranche.
3. Run
   `cargo test -p kamn-core --test kamn_core_decomposition_map_docs -- --nocapture`.
