# Spec: Issue #4454

Status: Implemented
Issue: #4454
Parent: #4448
Milestone: R27.39 Runtime decomposition, panic-free execution, and dependency-license governance
Priority: P1

## Problem Statement

Production panic reachability must fail closed before promotion. The existing production checker
covers `.expect(` but does not fully enforce panic-style path reachability (`panic!`,
`unreachable!`) or unsafe fallback-default behavior in production paths.

## Scope

In scope:
- RED/GREEN coverage for production panic-path reachability and unsafe fallback defaults.
- Checker updates required to satisfy the new RED coverage.
- Panic-reachability policy documentation in `docs/security/secure-coding.md`.

Out of scope:
- Test-only panic patterns under `#[cfg(test)]`.
- Runtime error-taxonomy normalization/output contracts (handled by follow-up #4455).

## Acceptance Criteria

AC-1:
Given production Rust files before `#[cfg(test)]` blocks, when panic-style paths are present,
then checker tests fail for `.expect(`, `panic!`, and `unreachable!` reachability.

AC-2:
Given production fallback-default behavior, when unsafe env-var fallback defaults are used,
then checker tests fail closed with deterministic violation output.

AC-3:
Given panic policy docs contracts, when docs tests run, then secure-coding docs include
panic-path reachability and unsafe-fallback failure-case references.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Test: `bash scripts/ci/test_check_no_production_expect.sh`
  - Expectation: panic-style production fixtures fail checker; test-only fixtures remain allowed.

- C-02 (AC-2, Functional/Conformance):
  - Test: `bash scripts/ci/test_check_no_production_expect.sh`
  - Expectation: unsafe fallback-default fixture fails checker.

- C-03 (AC-3, Regression/Conformance):
  - Test: `cargo test -p kamn-core --test secure_coding_docs`
  - Expectation: secure-coding doc contains panic reachability and unsafe fallback failure markers.

## Success Metrics / Observable Signals

- Checker fails closed for production panic-path and unsafe fallback fixtures.
- Existing allowed test-only paths remain non-violating.
- Secure-coding docs clearly encode required failure-case policy markers.
