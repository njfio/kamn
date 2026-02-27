# Issue 6230 Plan

## Approach
1. Extend the E2E workflow contract test (`e2e_live_workflow_lane`) with new fail-closed assertions for PR trigger, PR smoke routing, and bounded retry markers.
2. Run the updated contract test before workflow/doc changes to capture RED evidence.
3. Update `.github/workflows/e2e-live.yml`:
   - add `pull_request` trigger,
   - ensure PR path runs only CLI smoke (`S-01,S-02`),
   - wrap smoke execution in `scripts/ci/run_with_retry.sh --max-attempts 2`.
4. Update `docs/ci/strategy.md` E2E workflow contract markers to align with the expanded reason taxonomy.
5. Re-run the contract test to capture GREEN evidence and verify no drift.

## Affected Modules
- `.github/workflows/e2e-live.yml`
- `crates/kamn-core/tests/e2e_live_workflow_lane.rs`
- `docs/ci/strategy.md`
- `specs/6230/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: PR workflow cost increases too much.
  - Mitigation: run only CLI smoke scenarios `S-01,S-02` on PR; keep full matrix on non-PR lanes.
- Risk: flaky E2E failures create merge noise.
  - Mitigation: enforce bounded retry wrapper (`max-attempts 2`) and explicit contract markers.
- Risk: docs/test contract drift.
  - Mitigation: update deterministic reason-code taxonomy and strategy markers in same change.

## Interfaces
- CI workflow trigger/scope changes only.
- No runtime API/wire-format changes.
