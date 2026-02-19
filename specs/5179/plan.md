# Issue #5179 Plan

- Issue: #5179
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Approach
1. RED/Analysis:
   - Confirm signer lock usage currently relies on `.expect(...)` across signer tests.
   - Confirm ignored-test surface includes deep-lane mutation test.
   - Confirm PRD path references point to repository root path.
2. GREEN:
   - Add poison-tolerant lock helper and route signer tests through it.
   - Replace deep-lane ignore annotation with explicit env-gated skip.
   - Move PRD source under `docs/planning/` and patch references.
   - Refresh stale specs to Reviewed with dated staleness notes.
3. VERIFY:
   - Run targeted node and core test suites.
   - Run grep-based conformance checks for references/status markers.
   - Update issue process log and close acceptance criteria.

## Risks and Mitigations
- Risk: cross-module regressions from refactor touches.
  - Mitigation: keep diffs scoped and run targeted regressions.
- Risk: shell-surface growth or drift.
  - Mitigation: prefer Rust-first implementations; track deltas explicitly.
- Risk: stale issue/spec metadata divergence.
  - Mitigation: update issue comments and spec status markers in same change.

## Interfaces / Contracts
- Preserve existing public behavior unless spec explicitly changes it.
- Keep reason-code and contract marker semantics deterministic.
- Do not add production panic paths.
- Keep shell-surface delta neutral for this task.
