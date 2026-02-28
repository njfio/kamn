# Issue 6248 Plan

## Approach
1. Extend workflow contract tests to assert PR execution requirements for CLI, SDK-Direct, and MCP lanes.
2. Capture RED evidence from missing PR-lane assertions.
3. Update `.github/workflows/e2e-live.yml` to enforce required PR lanes and deterministic skip/fail semantics.
4. Update docs contract markers for PR E2E matrix.
5. Re-run contract and selected workflow-lane tests to verify GREEN behavior.

## Affected Modules
- `.github/workflows/e2e-live.yml`
- `crates/kamn-core/tests/e2e_live_workflow_lane.rs`
- `docs/ci/strategy.md`
- `docs/planning/r59-followup.md`
- `specs/6248/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: PR runtime cost grows beyond acceptable gate budget.
  - Mitigation: keep each lane smoke-scoped with bounded retries.
- Risk: secret/env availability differs by fork context.
  - Mitigation: deterministic reason-coded skip/fail policy with explicit contract checks.
- Risk: workflow/doc drift.
  - Mitigation: update and enforce contract marker assertions in Rust tests.

## Interfaces
- GitHub Actions workflow contract and CI docs.
- No runtime protocol or API contract changes.
