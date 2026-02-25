# Spec: Issue #5961 - Task: close escaped http_transport mutants from #5932 mutation gate

- Issue: #5961
- Status: Accepted (agent-authored + self-accepted under P2 single-module rule)
- Type: task
- Priority: P2
- Area: qa
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: #5932

## Problem Statement
Mutation testing for `crates/kamn-core/src/kolme_runtime_commit/http_transport.rs` reported 8 escaped mutants in transport equality and HTTP response parsing logic.

## Scope
In scope:
- Add targeted unit/regression tests that kill the 8 escaped mutants reported for issue #5932.
- Keep runtime behavior contract unchanged.

Out of scope:
- Functional redesign of HTTP transport.
- New transport features beyond mutant-closing test coverage.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: Equality semantics test coverage rejects all escaped comparison/equality mutants in `PartialEq`.
- AC-2: HTTP response read-path coverage rejects escaped control-flow and boundary-condition mutants in `read_http_response_bytes`.
- AC-3: HTTP metadata parse-path coverage rejects escaped control-flow mutant in `parse_http_response_metadata`.
- AC-4: Mutation rerun for the scoped file reports 0 escapes for the 8 listed mutants.

## Conformance Cases
- C-01 (Unit, AC-1): Transport equality depends on both timeout and authorization header values.
- C-02 (Unit, AC-2): Fragmented HTTP headers are tolerated until boundary is present.
- C-03 (Unit, AC-2): Content-length truncation happens only when full payload has been read.
- C-04 (Unit, AC-3): `Content-Length` parse is only applied to the matching header key.
- C-05 (Mutation, AC-4): Scoped mutation run for `http_transport.rs` has zero escapes for the eight reported mutation points.

## Success Metrics / Observable Signals
- All `spec_cxx_issue_5961_*` tests pass in `kamn-core` test runs.
- Scoped mutation output for `http_transport.rs` shows no escapes at previously escaped points.

## Required Test Categories
- Unit: targeted deterministic tests for equality and parsing paths.
- Conformance: test names map directly to C-01..C-04.
- Mutation: scoped run proving C-05.
- Regression: tagged tests for escaped mutant regressions.

## Dependencies
- #5932
- #5958
