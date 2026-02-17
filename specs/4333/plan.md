# Plan — #4333

Status: Reviewed

## Approach

- Extend shutdown contract validation in `runtime_orchestration.rs`:
  - enforce graceful drain/timeout consistency invariants,
  - parse and validate numeric reconciliation fields,
  - add checkpoint reconciliation classifier and validator with deterministic reason codes.
- Invoke reconciliation validation in daemon execution flow so full runtime inherits enforcement.
- Add unit/regression/integration tests for drift/fail-closed behavior.
- Update ops and release-go/no-go docs with shutdown reconciliation reason taxonomy markers.

## Affected Areas

- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/main_tests/runtime_tests.rs`
- `docs/ops/configuration.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations

- Risk: false positives if invariants depend on unknown max-tick context.
  - Mitigation: enforce only invariants derivable from encoded completion metadata and existing status classes.
- Risk: new reason taxonomy diverges from docs/tests.
  - Mitigation: add deterministic string assertions in runtime and docs contract tests.
