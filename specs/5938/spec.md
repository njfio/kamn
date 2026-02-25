# Spec: Issue #5938 - Task: Expand fuzzing and property-based testing across parser/protocol surfaces

- Issue: #5938
- Status: Implemented
- Type: task
- Priority: P1
- Area: qa
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5920

## Problem Statement
Current fuzz/proptest usage is minimal for system complexity.

## Scope
In scope:
- Add fuzz targets for key parser/protocol surfaces and broader proptest invariant suites.

Out of scope:
- Replacing deterministic regression suites with fuzz-only coverage.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Fuzz target set expands beyond current baseline with corpus tracking and CI/nightly execution.
- AC-2: Property tests cover key invariants for replay/auth/message/protocol logic.
- AC-3: Crash/panic regressions are reproducibly prevented via saved corpora and regression tests.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Fuzz target set expands beyond current baseline with corpus tracking and CI/nightly execution.
- C-02 (Functional, AC-2): Verify Property tests cover key invariants for replay/auth/message/protocol logic.
- C-03 (Functional, AC-3): Verify Crash/panic regressions are reproducibly prevented via saved corpora and regression tests.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: property invariants
- Functional: invariant-preserving behavior checks
- Integration: fuzz harnesses for wire/protocol boundaries
- Regression: corpus-derived repro tests
- Performance: fuzz iteration budget and corpus growth policy

## Dependencies
- #5920

## Implementation Notes
- Added two new fuzz targets: `signature_profile_parser` and `kolme_api_codec_parser`.
- Added deterministic seed corpora for both targets and expanded `cargo-fuzz-seed-corpus-v1.json`.
- Added parser/protocol property invariants in `crates/kamn-core/tests/parser_protocol_proptest_invariants.rs`.
- Added corpus replay regression checks in `crates/kamn-core/tests/cargo_fuzz_target_contract.rs`.
- Updated CI strategy/security/architecture docs and milestone index progress marker for R65.
