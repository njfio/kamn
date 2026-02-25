# Plan: Issue #5961 - Close escaped http_transport mutants from #5932 mutation gate

- Issue: #5961
- Spec: `specs/5961/spec.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Approach
1. RED: add focused tests in `http_transport.rs` test module for the 8 escaped decision points.
2. GREEN: keep implementation unchanged unless tests expose real behavior mismatch; adjust only assertions/fixtures as needed.
3. VERIFY: run targeted `kamn-core` tests for new coverage.
4. MUTATION: rerun mutation scope restricted to `http_transport.rs` and record totals.
5. PROCESS: publish evidence comments on #5957 and #5961.

## Affected Modules
- `crates/kamn-core/src/kolme_runtime_commit/http_transport.rs`
- `specs/5961/spec.md`
- `specs/5961/plan.md`
- `specs/5961/tasks.md`

## Risks + Mitigations
- Risk: mutation runtime is expensive.
  - Mitigation: scope mutation run to `http_transport.rs` escaped set.
- Risk: tests become overfitted to implementation details.
  - Mitigation: assert external behavior/contract signals (status parsing, completion semantics) not line structure.

## ADR Requirement
- Not required (test-surface hardening only).
