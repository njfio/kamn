# Spec: Issue 6475 - Document GitHub Actions outage recovery pattern

## Objective
Document the repository-standard response when pull requests lose GitHub Actions
checks because of a platform outage, so contributors verify GitHub Status first,
distinguish platform failures from branch regressions, and retrigger CI only
after Actions recovery.

## Inputs/Outputs
- Inputs:
  - `AGENTS.md`
  - `.github/CONTRIBUTING.md`
- Outputs:
  - Matching outage-recovery guidance in the compatibility and canonical
    contributor policy files.
  - A docs contract test that fails if the required outage markers disappear.

## Boundaries/Non-goals
- No workflow logic changes.
- No GitHub Actions configuration changes.
- No PR template or issue-template changes.

## Failure modes
- Contributors rewrite workflows instead of first checking GitHub Status during
  an Actions outage.
- Policy files drift and document different recovery instructions.
- The outage-recovery pattern disappears without test coverage.

## Acceptance criteria (testable booleans)
- [ ] AC-1: `AGENTS.md` instructs contributors to check GitHub Status when PR
      checks are missing or workflow dispatch returns platform-level failures.
- [ ] AC-2: `.github/CONTRIBUTING.md` documents the same GitHub Actions outage
      recovery pattern.
- [ ] AC-3: The documented recovery pattern includes a safe retrigger step after
      GitHub Actions recovery.
- [ ] AC-4: A docs contract test enforces the required outage-recovery markers
      in both policy files.
- [ ] AC-5: `cargo test -p kamn-core --test contributor_policy_ci_outage_docs`
      passes.

## Files to touch
- `specs/6475-document-github-actions-outage-recovery-pattern.md`
- `AGENTS.md`
- `.github/CONTRIBUTING.md`
- `crates/kamn-core/tests/contributor_policy_ci_outage_docs.rs`

## Error semantics
- Documentation changes only; no runtime error behavior changes.
- Missing required outage markers should fail the docs contract test loudly.

## Test plan
- Red:
  - Add a docs contract test that requires explicit outage-recovery markers in
    both policy files.
  - Confirm the test fails before the documentation is updated.
- Green:
  - Add the minimal policy guidance needed to satisfy the contract.
- Refactor:
  - Keep the policy language aligned between `AGENTS.md` and
    `.github/CONTRIBUTING.md`.
- Integration:
  - Run the dedicated docs contract target.

## Phase 6 integration evidence
- Pending.

## Deviations
- None.
