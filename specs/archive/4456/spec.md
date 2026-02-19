# Spec: Issue #4456

Status: Implemented
Issue: #4456
Parent: #4449
Milestone: R27.39 Runtime decomposition, panic-free execution, and dependency-license governance
Priority: P1

## Problem Statement

Dependency/license metadata drift and documentation mismatch must fail closed in CI. Missing or
under-scoped regression tests can allow drift acceptance and weaken release-governance guarantees.

## Scope

In scope:
- Red coverage for dependency posture docs drift acceptance cases.
- Red coverage for workspace license metadata drift acceptance cases.
- Release checklist docs contract coverage for dependency/license mismatch cases.

Out of scope:
- Introducing new dependency-governance reason taxonomy payload formats.
- CI lane architecture redesign.

## Acceptance Criteria

AC-1:
Given docs drift in dependency posture sources (README/CI strategy), when dependency posture checks
run, then failures are emitted with deterministic fail-closed reasons.

AC-2:
Given license metadata drift in Cargo manifests, when workspace license policy checks run, then
mismatches and malformed metadata fail closed.

AC-3:
Given release docs contracts, when docs tests run, then the release checklist includes dependency
/license mismatch acceptance guidance and regression markers.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Test: `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
  - Expectation: docs drift fixtures fail with deterministic reason markers.

- C-02 (AC-2, Unit/Regression/Conformance):
  - Test: `bash scripts/ci/test_check_workspace_license_policy.sh`
  - Expectation: metadata drift fixtures (mismatch/missing/malformed/package missing/not found)
    fail closed.

- C-03 (AC-3, Regression/Conformance):
  - Test: `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - Expectation: checklist contains dependency/license mismatch gate markers and regression policy.

## Success Metrics / Observable Signals

- Added red cases fail against drift scenarios and pass on baseline.
- Docs contract fails closed if checklist dependency/license mismatch markers are removed.
- Scoped CI checks remain deterministic and reproducible across runs.
