# Spec: Issue #5953 - Strengthen kamn-sdk service HTTPS mutation coverage

- Issue: #5953
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-25)
- Type: task
- Priority: P1
- Area: qa
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5918

## Problem Statement
Mutation testing for HTTPS transport paths in `kamn-sdk` reported surviving and timeout mutants, indicating insufficient test assertions around stream read/flush semantics and EOF behavior.

## Scope
In scope:
- Add targeted tests that fail when read/flush semantics are weakened.
- Add deterministic fail-fast behavior for pathological read loops and assert it in tests.
- Re-run scoped mutation suite for touched `kamn-sdk` transport paths.

Out of scope:
- Unrelated transport protocol redesign.
- Broad SDK API surface changes outside `service.rs` behavior under test.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: `ServiceStream::flush` behavior is covered by tests that fail when flush becomes a no-op success.
- AC-2: `read_response_bytes` EOF guard behavior is covered by tests that fail when unexpected-EOF handling is weakened.
- AC-3: Pathological short-read loops terminate deterministically with a bounded failure mode and are asserted by tests.
- AC-4: Scoped mutation run for touched `kamn-sdk` HTTPS paths has zero surviving and zero timeout mutants.

## Conformance Cases
- C-01 (Conformance, AC-1): Mutating flush semantics in `ServiceStream::flush` causes tests to fail.
- C-02 (Conformance, AC-2): Mutating unexpected-EOF guard in `read_response_bytes` causes tests to fail.
- C-03 (Conformance, AC-3): Replacing read progress with repeated `Ok(1)` cannot hang tests and produces deterministic failure.
- C-04 (Conformance, AC-4): `cargo mutants --in-diff` for issue scope reports no missed or timeout mutants.

## Success Metrics / Observable Signals
- New/updated tests in `crates/kamn-sdk/tests/service_api_client.rs` pass and are mutation-sensitive.
- Scoped mutation report for issue scope contains no `MISSED` and no `TIMEOUT` in touched paths.

## Required Test Categories
- Unit: required for transport helpers where applicable
- Functional: required for stream behavior assertions
- Conformance: required for C-01..C-04 mapping
- Regression: required for prior mutation escapes/timeouts
- Mutation: required and blocking for completion

## Dependencies
- #5918
