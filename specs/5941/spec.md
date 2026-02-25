# Spec: Issue #5941 - Task: Add cargo-audit dependency vulnerability scanning to required CI gates

- Issue: #5941
- Status: Implemented
- Type: task
- Priority: P1
- Area: devops
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5916

## Problem Statement
No dependency vulnerability scan currently blocks insecure dependency updates.

## Scope
In scope:
- Integrate cargo-audit into required CI with waiver policy and deterministic reporting.

Out of scope:
- Non-Rust dependency governance beyond current scope.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: CI fails on unwaived critical/high advisories.
- AC-2: Advisory exceptions require explicit tracked justification.
- AC-3: Security reports are archived for auditability.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify CI fails on unwaived critical/high advisories.
- C-02 (Functional, AC-2): Verify Advisory exceptions require explicit tracked justification.
- C-03 (Functional, AC-3): Verify Security reports are archived for auditability.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.

## Implementation Evidence (2026-02-25)
- AC-1: required CI jobs now run `cargo audit --json` and fail closed via `scripts/ci/check_cargo_audit_policy.py` when unwaived severity exceeds `moderate`.
- AC-2: waiver schema `.ci/cargo-audit-waivers.json` is enforced (`schema_version=kamn.ci.cargo-audit-waiver.v1`) with mandatory `reason`, `tracking_issue`, and non-expired `expires_on`.
- AC-3: workflows archive `cargo-audit-report.json` and `ci-cargo-audit-policy.json` artifacts for auditability.
- AC-4: checker regression suite added and passing:
  - `bash scripts/ci/test_check_cargo_audit_policy.sh`
  - `python3 -m py_compile scripts/ci/check_cargo_audit_policy.py`
  - `cargo fmt --check`


## Required Test Categories
- Unit: CI script policy checks
- Functional: advisory classification behavior
- Integration: workflow run with failing advisory fixture
- Regression: policy remains enforced across workflow updates
- Performance: CI overhead documented

## Dependencies
- #5916
