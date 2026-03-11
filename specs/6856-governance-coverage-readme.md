## Objective

Increase `kamn-governance` coverage with meaningful lifecycle, quorum, voting, and fail-closed tests, and add the missing crate README so the crate meets the same onboarding baseline as the rest of the workspace.

## Inputs/Outputs

Inputs:
- Existing governance workflow public API in `crates/kamn-governance/src/governance_workflow`
- Existing operator binding and operator action public APIs in `crates/kamn-governance/src/operator_binding` and `crates/kamn-governance/src/operator_actions`
- Current crate metadata in `crates/kamn-governance/Cargo.toml`

Outputs:
- Additional governance workflow tests covering approval, expiration, duplicate-vote rejection, execute fail-closed behavior, and parameter-policy failures
- Additional operator binding/action tests covering duplicate binding, revoked authorization denial, unauthorized configure denial audit, and successful audit history reads
- `crates/kamn-governance/README.md` with crate purpose, exported surfaces, and local test entrypoints
- A contract test that fails if the README is removed or loses required sections

## Boundaries/Non-goals

Non-goals:
- Rewriting governance workflow behavior
- Changing public governance APIs
- Inflating test counts with trivial assertions
- Broad crate graph or dependency changes unrelated to governance coverage and README presence

Boundaries:
- Keep changes scoped to `crates/kamn-governance/**`
- Reuse current public APIs and current test harnesses
- New tests must exercise real in-memory workflow/binding/action logic, not mocks

## Failure Modes

- Governance proposal approval path regresses and no test catches terminal `Approved` state
- Proposal expiration or post-deadline vote rejection regresses silently
- Duplicate-vote or duplicate-binding fail-closed behavior regresses
- Revoked or unauthorized operator requests stop emitting denied outcomes correctly
- README is missing or drifts away from minimum crate purpose/usage guidance

## Acceptance Criteria

- [ ] `kamn-governance` has new tests covering governance approval/quorum success
- [ ] `kamn-governance` has new tests covering expiry and fail-closed vote/execute paths
- [ ] `kamn-governance` has new tests covering duplicate binding, revoked authorization, and denied operator action audit behavior
- [ ] `crates/kamn-governance/README.md` exists and includes purpose, exported surfaces, and local test guidance
- [ ] A contract test enforces the README markers
- [ ] `cargo test -p kamn-governance -- --nocapture` passes

## Files To Touch

- `crates/kamn-governance/README.md`
- `crates/kamn-governance/tests/governance_workflow_internal.rs`
- `crates/kamn-governance/src/operator_binding/tests.rs`
- `crates/kamn-governance/src/operator_actions/tests.rs`
- `crates/kamn-governance/tests/governance_readme_contract.rs`

## Error Semantics

- Tests must assert the current typed error variants and fail-closed statuses already exposed by the crate
- README contract failures should use direct, explicit assertions with actionable missing-marker messages
- No production error semantics should be weakened or changed in this issue

## Test Plan

Red:
- Add governance workflow tests for approval, expiry, duplicate vote, execute-before-approval rejection, and parameter policy failure
- Add operator binding/action tests for duplicate binding, revoked authorization denial, denied configure audit logging, and successful history reads
- Add a README contract test that fails until `crates/kamn-governance/README.md` exists with required markers

Green:
- Add the README
- Adjust or extend tests only enough to align with existing runtime behavior

Refactor/Integration:
- Keep each test file under the active size limits
- Run `cargo test -p kamn-governance -- --nocapture`

## Integration Evidence

- `crates/kamn-governance/README.md` now exists as the crate-local onboarding entrypoint
- The new governance coverage is exercised through the real `cargo test -p kamn-governance -- --nocapture` crate target
- The README contract is enforced by `crates/kamn-governance/tests/governance_readme_contract.rs`
- The touched-Rust size policy passes on the governance coverage write set via `check_touched_rust_size_policy.py`

## Deviations

- No production entrypoint wiring changed because this issue only expands crate-local coverage and documentation
