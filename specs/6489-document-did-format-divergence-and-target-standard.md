## Objective

Document the current DID format divergence in the repository and record the target canonical
format before any public-contract or parser behavior changes are attempted.

## Inputs/Outputs

- Inputs:
  - existing repo usages of `kamn:did:...`
  - existing repo usages of `did:kamn:...`
  - architecture/docs contract tests
- Outputs:
  - one architecture document that inventories current DID format divergence with concrete examples
  - explicit target canonical DID format and non-goals for the planning slice
  - contract coverage that fails if the required documentation markers disappear

## Boundaries/Non-goals

- No production DID parsing behavior changes
- No migration of existing call sites
- No CI/workflow changes
- No attempt to standardize runtime behavior in this issue

## Failure modes

- The documented target standard is ambiguous
- The current divergent `did:kamn:...` consumers are not explicitly identified
- The planning document silently disappears or drifts without test coverage

## Acceptance criteria

- [ ] A document inventories both canonical `kamn:did:...` and divergent `did:kamn:...` shapes with concrete repo examples
- [ ] The document states the target canonical format for future implementation work
- [ ] The document records non-goals for the planning slice and follow-up implementation direction
- [ ] A docs contract test pins the required DID divergence markers
- [ ] `cargo test -p kamn-types --test identity_boundary_contract -- --nocapture` passes

## Files to touch

- `specs/6489-document-did-format-divergence-and-target-standard.md`
- `docs/architecture/did-format-standardization.md`
- `docs/architecture/kamn-types.md`
- `crates/kamn-types/tests/identity_boundary_contract.rs`

## Error semantics

- No runtime error semantics change in this issue
- This issue only documents the currently divergent DID shapes and the intended canonical target

## Test plan

- Extend the existing `kamn-types` identity boundary contract test to require markers in the new
  DID format standardization document
- Add/update architecture documentation with concrete examples of the two DID shapes
- Run:
  - `cargo test -p kamn-types --test identity_boundary_contract -- --nocapture`

## Deviations

- None
