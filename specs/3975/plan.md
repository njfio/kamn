# Plan — Issue #3975

## Approach

1. Extend `check_kamn_core_missing_docs_policy.sh` reason-code CSV and reuse `fail_with_reason` for graduated-module exemption regression failure path.
2. Update `test_check_kamn_core_missing_docs_policy.sh`:
   - reason-code CSV expectation for rustdoc-link drift path.
   - new assertions for graduated-module exemption regression reason markers.
3. Update docs marker references:
   - `docs/ci/strategy.md`
   - `docs/architecture/runtime.md`
4. Update docs-contract assertions in `crates/kamn-core/tests/runtime_architecture_docs.rs`.
5. Run targeted tests and fast CI tools regression.

## Affected Paths

- `scripts/ci/check_kamn_core_missing_docs_policy.sh`
- `scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
- `docs/ci/strategy.md`
- `docs/architecture/runtime.md`
- `crates/kamn-core/tests/runtime_architecture_docs.rs`
- `specs/3975/spec.md`
- `specs/3975/plan.md`
- `specs/3975/tasks.md`

## Risks / Mitigations

- Risk: Existing docs/test strings for old single-code CSV drift and break.
  Mitigation: update checker/docs/tests in same change and run targeted docs contract tests + fast CI regression.

- Risk: unintended marker changes in unrelated failure paths.
  Mitigation: scope reason-marker emission update only to exemption regression path and existing rustdoc parity path behavior.

## ADR

- Not required (policy-check contract marker update only).
