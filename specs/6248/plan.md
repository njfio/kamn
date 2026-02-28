# Issue 6248 Plan

## Approach
1. Extend the E2E workflow contract test to enforce PR lane scope for SDK and MCP, plus deterministic PR skip markers.
2. Capture RED evidence from the updated contract test against the current workflow/doc baseline.
3. Update `.github/workflows/e2e-live.yml`:
   - enable SDK and MCP jobs for PR events,
   - add PR smoke scenario selectors,
   - emit deterministic PR skip-reason markers.
4. Update `docs/ci/strategy.md` E2E live workflow contract markers.
5. Re-run contract tests for GREEN evidence.

## Affected Modules
- `.github/workflows/e2e-live.yml`
- `crates/kamn-core/tests/e2e_live_workflow_lane.rs`
- `docs/ci/strategy.md`
- `specs/6248/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: PR runtime overhead rises due additional lanes.
  - Mitigation: PR scope uses bounded smoke scenario slices (`S-01,S-02`) for SDK/MCP/CLI.
- Risk: lane drift between workflow and docs.
  - Mitigation: fail-closed contract test requires both workflow markers and strategy markers.

## Interfaces
- GitHub Actions workflow contract surface.
- CI strategy documentation contract markers.
