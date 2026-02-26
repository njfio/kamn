# Plan: Issue #6087

## Approach
1. Add Red checks to `scripts/ci/test_workflow_scope_policy.sh` for:
   - panic-surface gate step marker,
   - checker command marker,
   - report artifact marker.
2. Run workflow policy script to capture Red failure before workflow wiring exists.
3. Update `.github/workflows/ci-fast-gate.yml` to:
   - run checker in Rust scope,
   - upload report artifact when present.
4. Re-run workflow policy script to Green and validate checker command locally.

## Affected Modules
- `.github/workflows/ci-fast-gate.yml`
- `scripts/ci/test_workflow_scope_policy.sh`
- `specs/milestones/r67-runtime-hardening-and-surface-reduction/index.md`

## Risks / Mitigations
- Risk: workflow step added without stable contract checks.
  Mitigation: add explicit grep-based contract assertions in `test_workflow_scope_policy.sh`.
- Risk: report artifact wiring drifts.
  Mitigation: include file-name marker assertion and artifact upload block assertion.

## Interfaces / Contracts
- Fast-gate workflow step contract for production panic-surface checker.
- CI artifact contract for `ci-no-production-expect-report.json`.
