## Objective

Stabilize the `E2E SDK-Direct` live validation path on current `main` by fixing the concrete `S-14` failure without weakening the live harness validation contract.

## Inputs/Outputs

Inputs:
- Current `main` failure from GitHub Actions run `22961536343`, job `E2E SDK-Direct`
- Real external-live-stack SDK-direct harness path in `crates/kamn-e2e-harness/**`
- Existing `S-14` scenario and validation logic used by the live harness

Outputs:
- A concrete fix for the `S-14` live validation failure on current `main`
- Regression coverage or contract coverage tied to the real failing path
- Updated spec evidence documenting the exact failure and fix

## Boundaries/Non-goals

Non-goals:
- Weakening, bypassing, or disabling the E2E live validation gate
- Rewriting unrelated scenarios outside `S-14`
- Replacing the live path with mocks

Boundaries:
- Keep changes scoped to the SDK-direct live scenario/validation path and directly related supporting code/tests/specs
- Preserve the existing external-live-stack execution model

## Failure modes

- `S-14` scenario still fails under the real live harness path
- The harness reports `validation_status=FAIL` or `overall_status=FAIL` after the fix
- The fix masks the problem by loosening validation instead of correcting behavior
- A regression in `S-14` is not covered by a repeatable issue-scoped test/contract

## Acceptance criteria

- [ ] The `S-14` SDK-direct live failure is reproduced from current `main` or an issue-scoped equivalent based on the same real path
- [ ] The concrete failing assertion or runtime contract is identified and documented in the spec
- [ ] The underlying cause is fixed without weakening the live validation contract
- [ ] The repaired branch passes the real `E2E SDK-Direct` harness path for the affected scenario set
- [ ] Regression coverage is added or tightened around the exact `S-14` failing path

## Files to touch

- `specs/6879-stabilize-sdk-direct-s14-live-validation.md`
- `crates/kamn-e2e-harness/**` only as required by the real failing path
- related workflow/test harness files only if directly required by the repro or validation surface

## Error semantics

- Keep current hard-fail validation behavior for live execution results
- Do not convert failing live checks into warnings or tolerated drift
- Any new failure messaging should remain explicit about the violated live contract

## Test plan

Red:
- Reproduce the `S-14` failure via the real SDK-direct live harness path or a narrow issue-scoped regression test derived from it
- Add or tighten regression coverage so the current broken behavior fails

Green:
- Implement the minimal fix required for `S-14` to satisfy the existing live contract

Refactor/Integration:
- Keep touched files/functions within active size policy
- Re-run the issue-scoped live/contract checks
- Record the exact evidence and any deviations in this spec
