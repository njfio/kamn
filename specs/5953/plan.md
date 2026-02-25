# Plan: Issue #5953 - Strengthen kamn-sdk service HTTPS mutation coverage

- Issue: #5953
- Spec: `specs/5953/spec.md`
- Status: Draft
- Last Updated: 2026-02-25

## Approach
1. RED: Add/adjust tests in `crates/kamn-sdk/src/service.rs` and `crates/kamn-sdk/tests/service_api_client.rs` that expose the previously surviving/timeout mutants.
2. Implement: Add bounded response-read safeguards in `crates/kamn-sdk/src/service.rs` so pathological reads fail fast deterministically.
3. REGRESSION: Run targeted `kamn-sdk` tests covering `ServiceStream` and response reading semantics.
4. VERIFY: Run scoped mutation for issue diff and confirm no `MISSED`/`TIMEOUT` mutants in touched paths.

## Affected Modules (Initial)
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/tests/service_api_client.rs`

## Risks + Mitigations
- Risk: Read-loop safeguards can inadvertently reject valid slow streams.
  - Mitigation: bound by no-progress iterations and reset counters on progress.
- Risk: Mutation sensitivity may remain weak due to indirect assertions.
  - Mitigation: add direct assertions on flush propagation, EOF handling, and bounded failure conditions.

## Interfaces / Contracts
- Binding contract: `specs/5953/spec.md`
- Parent contract: issue #5918 and `specs/5918/spec.md`

## ADR Requirement
- No ADR expected unless implementation introduces new dependencies or transport contract changes.
