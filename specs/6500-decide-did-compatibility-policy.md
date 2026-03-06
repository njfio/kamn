## Objective
Record the repository policy for legacy `did:kamn:...` inputs before any parser, API, or runtime
behavior changes are attempted.

## Inputs/Outputs
- Inputs:
  - `docs/architecture/did-format-standardization.md`
  - `crates/kamn-types/tests/identity_boundary_contract.rs`
- Outputs:
  - explicit policy markers for legacy-input handling
  - documented boundary for where the policy applies
  - contract coverage that pins the documented policy

## Boundaries/Non-goals
- Do not change parser, runtime, API, CLI, or SDK behavior.
- Do not implement compatibility shims or reject paths in code.
- Do not modify CI/workflow/shell surfaces.

## Failure modes
- The repository does not say whether legacy `did:kamn:...` inputs should be tolerated or
  rejected.
- The policy boundary is ambiguous, so future implementation issues can enforce different scopes.
- The documented policy can drift without any failing contract test.

## Acceptance criteria
- [ ] `docs/architecture/did-format-standardization.md` defines the legacy-input policy with
      machine-readable markers.
- [ ] The doc states whether the policy is a temporary compatibility window or direct fail-closed
      rejection.
- [ ] The doc identifies the intended enforcement boundary.
- [ ] The doc states the implementation preconditions/gate for any future enforcement issue.
- [ ] `crates/kamn-types/tests/identity_boundary_contract.rs` pins the new policy markers.
- [ ] Focused policy doc contract tests pass locally.

## Files to touch
- `docs/architecture/did-format-standardization.md`
- `crates/kamn-types/tests/identity_boundary_contract.rs`
- `specs/6500-decide-did-compatibility-policy.md`

## Error semantics
- No runtime or parser error semantics change in this issue.
- This issue only documents the intended future behavior and enforcement scope.

## Test plan
- Red:
  - extend `identity_boundary_contract` so it requires the new policy markers
  - run the focused contract test and confirm it fails before the doc is updated
- Green:
  - `cargo test -p kamn-types --test identity_boundary_contract -- --nocapture`
- Refactor:
  - rerun the focused contract test after wording/marker cleanup
