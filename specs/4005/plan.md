# Issue #4005 Plan

## Approach

1. Establish RED coverage in `crates/kamn-core/tests/observability_stack_docs.rs` for missing capacity dry-run taxonomy markers and fail-closed regression marker.
2. Update `docs/foundation/observability-slo-dashboards.md` with a dedicated capacity policy-checker taxonomy section that includes deterministic marker strings and regression contract text.
3. Re-run targeted docs + governance contract suites to verify AC-to-conformance mapping remains satisfied.

## Affected Modules

- `docs/foundation/observability-slo-dashboards.md`
- `crates/kamn-core/tests/observability_stack_docs.rs`
- `specs/4005/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations

- Risk: docs markers drift from the checker threshold fixture.
  - Mitigation: assert exact marker strings in docs contract tests.
- Risk: scope creep into checker behavior already delivered under parent task/subtasks.
  - Mitigation: keep code changes constrained to docs + docs-contract assertions.
- Risk: test runtime expansion.
  - Mitigation: run targeted suites only for this change scope.

## Interfaces and Contracts

- No new dependencies.
- No schema/wire changes.
- Existing checker reason taxonomy contract remains:
  - `kamn.runtime.capacity-ci-dry-run-governance-reason-taxonomy.v1`
