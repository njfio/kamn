## Objective
Add an explicit `kamn-types` leaf-extraction tranche to the canonical `kamn-core` decomposition
roadmap so the identity/types seam is planned in the same tracked surface as the other extraction
targets.

## Inputs/Outputs
- Inputs:
  - `docs/architecture/kamn-core-module-map.md`
  - `crates/kamn-core/tests/kamn_core_decomposition_map_docs.rs`
  - `docs/architecture/README.md`
- Outputs:
  - updated decomposition roadmap markers that include `kamn-types`
  - explicit tranche row describing the identity/types extraction boundary and ordering rationale
  - hotspot wording aligned with the now-tracked `kamn-types` tranche

## Boundaries/Non-goals
- Do not perform any production extraction or crate wiring.
- Do not change runtime, parser, or API behavior.
- Do not add dependencies or modify CI/workflows.
- Do not redesign the full roadmap beyond the minimal sequencing needed to track `kamn-types`.

## Failure modes
- `kamn-types` remains only an implicit hotspot follow-up and not a tracked roadmap tranche.
- Roadmap marker values drift from the actual tranche table row count and target crates.
- The hotspot table still points to an untracked `kamn-types` follow-up after the roadmap update.
- Architecture index linkage regresses.

## Acceptance criteria
- [x] The roadmap table includes an explicit `kamn-types` extraction tranche.
- [x] `kamn_core_decomposition_tranche_count` and `kamn_core_decomposition_target_crates_csv`
      include the new `kamn-types` tranche.
- [x] The hotspot table no longer relies on an implicit `kamn-types` follow-up outside the tracked
      tranche roadmap.
- [x] Existing docs contracts and architecture index linkage pass.

## Files to touch
- `docs/architecture/kamn-core-module-map.md`
- `crates/kamn-core/tests/kamn_core_decomposition_map_docs.rs`
- `specs/6516-add-kamn-types-decomposition-tranche.md`

## Error semantics
- No runtime error semantics change in this issue.
- Contract failures should remain deterministic via the existing docs contract test.

## Test plan
- Red:
  - extend the existing decomposition map docs contract to require the new tranche count and target
    crate list including `kamn-types`
  - confirm the contract fails before the roadmap doc is updated
- Green:
  - `cargo test -p kamn-core --test kamn_core_decomposition_map_docs -- --nocapture`
- Refactor:
  - rerun the same focused docs contract after wording cleanup

## Deviations
- None.

## Execution Evidence
- Red:
  - `cargo test -p kamn-core --test kamn_core_decomposition_map_docs -- --nocapture`
- Green:
  - `cargo test -p kamn-core --test kamn_core_decomposition_map_docs -- --nocapture`
- Refactor / Integration:
  - `cargo fmt --all --check`
  - `cargo test -p kamn-core --test kamn_core_decomposition_map_docs -- --nocapture`
