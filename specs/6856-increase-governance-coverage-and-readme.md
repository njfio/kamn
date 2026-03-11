# 6856 — Increase kamn-governance coverage and add crate README

## Objective
Verify the current `kamn-governance` baseline on `origin/main` and expand meaningful fail-closed and lifecycle coverage where the crate is still thinly exercised. Because `crates/kamn-governance/README.md` already exists on the verified baseline, this issue will treat README work as verification/update-only rather than creating a missing file.

## Inputs/Outputs
### Inputs
- Current `origin/main` `kamn-governance` source and test surface
- Existing `README.md` and README contract
- Current AGENTS.md size and touched-Rust policy

### Outputs
- Additional meaningful governance workflow tests for parameter-policy and lifecycle/fail-closed paths
- Additional meaningful operator-action tests for denied/revoked/read-history fail-closed behavior
- Updated README only if verification shows it is inaccurate or incomplete
- Spec evidence documenting the baseline mismatch against the stale issue wording

## Boundaries / Non-goals
- No governance behavior rewrites
- No artificial test-count inflation without behavior assertions
- No crate-graph changes unrelated to governance coverage
- No weakening of existing tests or README contract checks

## Failure modes
- Added tests only verify file presence or marker strings without exercising behavior
- README is changed unnecessarily even though the current baseline is already accurate
- New tests pass trivially and fail to cover fail-closed governance behavior
- Refactor or test helper changes violate touched-Rust size policy
- Baseline mismatch (README already present) is not recorded in the spec

## Acceptance criteria
- [x] Verified current `origin/main` baseline for `kamn-governance` is recorded in this spec, including the fact that `README.md` already exists
- [x] Add meaningful workflow tests covering currently under-tested parameter-policy and governance lifecycle/fail-closed paths
- [x] Add meaningful operator-action tests covering denied/revoked/read-history fail-closed behavior
- [x] Existing governance behavior remains stable under full crate test runs
- [x] Touched-Rust size policy returns `policy_decision=GO`
- [x] README is either verified accurate as-is or updated narrowly with evidence recorded in this spec

## Files to touch
- `specs/6856-increase-governance-coverage-and-readme.md`
- `crates/kamn-governance/tests/`
- `crates/kamn-governance/README.md` only if verification proves it needs update

## Error semantics
- New tests must assert exact fail-closed error variants for rejected governance/operator actions
- No silent fallbacks or weakened assertions
- README verification mismatches must be recorded explicitly in the spec

## Test plan
1. Add a red contract asserting the intended new governance coverage surface exists
2. Confirm the contract fails on current `origin/main`
3. Add real behavior tests for parameter-policy and operator-action fail-closed paths
4. Run:
   - `cargo test -p kamn-governance -- --nocapture`
   - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root <clean-clone> --base-ref origin/main --output-json <file>`
5. Record baseline mismatch and final evidence in this spec

## Verified current-main state
- `crates/kamn-governance/README.md` already exists on the verified baseline
- Current test surface is materially better than the stale issue wording suggested
- Remaining legitimate gap is deeper fail-closed coverage around parameter-policy and operator-action paths rather than README creation itself

## Phase 4 Green Evidence
- Added focused behavior tests:
  - `crates/kamn-governance/tests/governance_parameter_policy_fail_closed.rs`
  - `crates/kamn-governance/tests/operator_actions_fail_closed.rs`
- Added hard-fail coverage contract:
  - `crates/kamn-governance/tests/governance_coverage_expansion_contract.rs`
- New behavior coverage includes:
  - unsupported target-version rejection for governance parameter changes
  - out-of-bounds parameter proposal rejection
  - denied read-history audit recording
  - denied revoke-binding audit recording

## Phase 5 Refactor Evidence
- Reduced duplication in the new tests with focused helper builders for:
  - parameter-change draft creation
  - denied audit assertion
  - shared DID constants in operator-action tests
- Verified:
  - `cargo test -p kamn-governance -- --nocapture`
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-6856-clean-tjrtha --base-ref origin/main --output-json /tmp/6856-touched-size-refactor.json`
- Final touched-Rust result: `policy_decision=GO`

## Phase 6 Integration Evidence
- The new tests execute through the real exported crate APIs:
  - `GovernanceWorkflow`
  - `PermissionedOperatorActionService`
  - `OperatorBindingEngine`
- Existing README contract remains green without README modification:
  - `cargo test -p kamn-governance governance_readme_contract -- --nocapture`

## Deviations
- The original issue wording claimed `kamn-governance` was missing a `README.md`; this was false on the verified baseline, so the issue was executed as coverage expansion plus README verification instead of README creation.
