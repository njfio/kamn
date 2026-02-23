# Plan: Issue #5815 - Close Residual S-03 sdk-direct Mutation Escapes

- Issue: #5815
- Status: Completed
- Spec: `specs/5815/spec.md`

## Approach
1. Extract S-03 response-shape validation into a small pure helper that checks:
   - queried message_id equality
   - listed channel_id equality
2. Add RED unit tests against that helper for mismatch cases.
3. Wire helper into `run_live_s03_group_channel_probe` without behavior change.
4. Run targeted tests, full harness regression, and mutation gate for diff.

## Affected Artifacts
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `specs/5815/spec.md`
- `specs/5815/plan.md`
- `specs/5815/tasks.md`

## Risks and Mitigations
- Risk: helper extraction changes runtime error text.
  - Mitigation: preserve existing error string shapes used by current checks.
- Risk: mutation lane flakes on unrelated tests.
  - Mitigation: run `--baseline skip` and record results; keep scoped diff small.

## Verification Strategy
- RED: new mismatch tests fail before helper/wiring.
- GREEN: tests pass after helper/wiring.
- Mutation: `cargo mutants --in-diff` catches previously escaped sdk-direct S-03 mutants.
