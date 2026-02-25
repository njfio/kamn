# Plan: Issue #5961 - Task: close escaped http_transport mutants from #5932 mutation gate

- Issue: #5961
- Spec: `specs/5961/spec.md`
- Status: Draft
- Last Updated: 2026-02-25

## Approach
1. RED: add spec-derived tests for equality and response parsing behavior tied to escaped mutants.
2. GREEN: adjust tests/helpers only (no behavior redesign) until new tests pass.
3. REGRESSION: run `cargo test -p kamn-core kolme_runtime_commit::http_transport`.
4. VERIFY: run scoped mutation for touched module and confirm escaped mutant count reaches 0 for the 8 known points.

## Affected Modules
- `crates/kamn-core/src/kolme_runtime_commit/http_transport.rs`

## Risks + Mitigations
- Risk: tests using sockets can become flaky.
  - Mitigation: use loopback listener with deterministic payload ordering and short thread joins.
- Risk: mutation reruns can be time-consuming.
  - Mitigation: scope mutation run to touched file/function where supported and include deterministic evidence logs.

## Interfaces / Contracts
- No API or wire-format changes.
- Contract source: `specs/5961/spec.md` and issue #5961 mutant list.

## ADR Requirement
- None expected (no dependency/protocol/architecture change).
