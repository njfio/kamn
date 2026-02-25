# Spec: Issue #5933 - Task: Decompose kamn-core into focused crates with phase-1 extraction

- Issue: #5933
- Status: Implemented
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5919

## Problem Statement
kamn-core currently concentrates many domains, increasing change risk and compile blast radius.

## Scope
In scope:
- Extract first tranche into focused crates with stable boundaries and minimal API breakage.

Out of scope:
- Total architecture rewrite in one cycle.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: A documented extraction plan is implemented for phase-1 crate split.
- AC-2: Moved modules compile and pass with preserved behavior contracts.
- AC-3: Public API boundaries and dependency graph are updated and documented.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify A documented extraction plan is implemented for phase-1 crate split.
- C-02 (Functional, AC-2): Verify Moved modules compile and pass with preserved behavior contracts.
- C-03 (Functional, AC-3): Verify Public API boundaries and dependency graph are updated and documented.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: moved module suites stay green
- Functional: cross-crate behavior parity
- Integration: workspace builds/tests with new crate graph
- Regression: API breakage checks for extracted surfaces
- Performance: compile-time telemetry captured

## Dependencies
- #5919

## Implementation Summary
- Added focused crate: `crates/kamn-runtime-guards`.
- Extracted modules: `anti_spam`, `fairness_policy`, `quota_policy`,
  `message_delivery_guards`, `retention_engine`, `watchdog`.
- Preserved `kamn-core` API paths via compatibility shims:
  `crates/kamn-core/src/{anti_spam,fairness_policy,quota_policy,message_delivery_guards,retention_engine,watchdog}.rs`.
- Documented crate-boundary change and ADR:
  `docs/architecture/adr-002-runtime-guards-phase1-extraction.md`.

## AC Verification
- AC-1 / C-01: `specs/5933/plan.md` executed with RED->GREEN extraction proof via
  `cargo test -p kamn-core --test issue_5933_runtime_guards_extraction`.
- AC-2 / C-02: `cargo test -p kamn-runtime-guards` and
  `cargo test -p kamn-core --test issue_5933_runtime_guards_extraction` pass.
- AC-3 / C-03: Dependency graph/docs updated in `Cargo.toml`,
  `crates/kamn-core/Cargo.toml`, and `docs/architecture/kamn-core-module-map.md`.
- AC-4 / C-04: Unit/functional/integration/regression coverage present and passing:
  `cargo test -p kamn-runtime-guards`,
  `cargo test -p kamn-core --test issue_5933_runtime_guards_extraction`,
  `cargo clippy -p kamn-core --tests -- -D warnings`,
  `cargo fmt --check`.
