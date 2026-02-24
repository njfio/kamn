# Plan: Issue #5857

## Approach
1. Add push trigger for `main` to `.github/workflows/e2e-live.yml`.
2. Keep run-cost bounded by making SDK-direct the guaranteed push lane; retain existing schedule/manual behavior for other jobs unless needed for consistency.
3. Extend workflow contract tests in `crates/kamn-core/tests/e2e_live_workflow_lane.rs` with new reason codes and regression mutations for push-trigger drift.
4. Extend phase-4i workflow contract tests in `crates/kamn-e2e-harness/tests/phase4i_ci_workflow_contract.rs` to assert push/main markers directly.
5. Update `docs/ci/strategy.md` E2E live workflow marker taxonomy to match updated test constants.
6. Run targeted tests and formatting/lint checks for touched surfaces.

## Affected Modules
- `.github/workflows/e2e-live.yml`
- `crates/kamn-core/tests/e2e_live_workflow_lane.rs`
- `crates/kamn-e2e-harness/tests/phase4i_ci_workflow_contract.rs`
- `docs/ci/strategy.md`
- `specs/5857/spec.md`
- `specs/5857/plan.md`
- `specs/5857/tasks.md`

## Risks & Mitigations
- Risk: push-trigger expansion may increase CI cost.
  - Mitigation: keep required push lane scoped to SDK-direct job and preserve schedule-only lane for heavier jobs where appropriate.
- Risk: taxonomy drift between code and docs.
  - Mitigation: centralize constants in workflow contract tests and update strategy markers in the same commit.
- Risk: flaky semantics from workflow syntax drift.
  - Mitigation: add regression mutation tests for trigger marker removal.

## Interfaces / Contracts
- E2E live workflow trigger contract:
  - push trigger scoped to `main`
  - schedule trigger retained
  - workflow_dispatch retained
- SDK-direct live lane contract:
  - external process execution enabled
  - Kolme local bootstrap + health waits enforced
  - full scenario matrix S-01..S-15 retained

## ADR
- Not required (workflow/test contract hardening; no dependency/protocol architecture change).
