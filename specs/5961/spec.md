# Spec: Issue #5961 - Close escaped http_transport mutants from #5932 mutation gate

- Issue: #5961
- Status: Implemented
- Type: task
- Priority: P2
- Area: qa
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: #5932

## Problem Statement
Mutation evidence for #5932 reported 8 escaped mutants in `crates/kamn-core/src/kolme_runtime_commit/http_transport.rs`, indicating missing assertions around equality semantics and response parsing branch behavior.

## Scope
In scope:
- Add focused tests that fail for the 8 escaped mutants listed in #5961.
- Keep production runtime behavior unchanged.
- Re-run mutation scope for `http_transport.rs` and demonstrate these mutants are caught.

Out of scope:
- Broad refactors unrelated to escaped mutant points.

## Acceptance Criteria
- AC-1: New tests fail against each of the 8 escaped mutant transformations and pass on canonical code.
- AC-2: Mutation rerun for `http_transport.rs` shows zero escaped mutants for the #5961 escaped set.
- AC-3: Evidence is posted on #5957 and linked from #5961.

## Conformance Cases
- C-01 (Conformance, AC-1): `PartialEq` behavior tests detect constant-true/constant-false and comparator-flip mutations in transport equality logic.
- C-02 (Conformance, AC-1): response parser tests detect relaxed header/content invariants (`&& -> ||`, `>= -> <`) in `read_http_response_bytes` and `parse_http_response_metadata` paths.
- C-03 (Conformance, AC-2): targeted mutation run on `http_transport.rs` reports zero missed mutants for the #5961 set.
- C-04 (Functional, AC-3): PR evidence comment includes before/after escaped counts and links.

## Success Metrics / Observable Signals
- Escaped count for the #5961 mutant set drops from 8 to 0.
- No regressions in existing `kamn-core` tests for runtime commit transport behavior.

## Implementation Evidence (2026-02-25)
- AC-1: Added regression tests in `crates/kamn-core/tests/kolme_runtime_commit_http_transport.rs`:
  - `regression_http_transport_partial_eq_requires_timeout_and_authorization_match`
  - `regression_http_transport_parses_connection_header_before_content_length`
  - `regression_http_transport_parses_chunked_headers_without_early_failure`
- AC-2: Targeted mutant rerun command:
  - `CARGO_BUILD_JOBS=1 cargo mutants --in-place -p kamn-core -f crates/kamn-core/src/kolme_runtime_commit/http_transport.rs -F 'http_transport\\.rs:(50:9|51:13|50:30|51:42|291:17|301:45|336:60)'`
  - Result: `8 mutants tested in 11m: 8 caught` (`missed=0`, `unviable=0`, `timeout=0`)
- AC-3: Mutation evidence is prepared for posting on PR/issue threads (`#5957`, `#5961`) with command, totals, and caught-mutant list.
