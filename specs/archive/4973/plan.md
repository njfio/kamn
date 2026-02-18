# Issue #4973 Plan

- Issue: #4973
- Status: Implemented

## Approach (Implemented)
1. Added dedicated governance doc for archive layout and retention policy markers.
2. Added marker presence assertions to existing spec-archive policy contract test.
3. Linked policy doc marker into the milestone governance marker section.
4. Re-ran archive checker/contract regression checks.

## Affected Modules
- `docs/planning/spec-archive-policy.md`
- `scripts/ci/test_check_spec_archive_policy.sh`
- `specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md`
- `specs/4973/spec.md`
- `specs/4973/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigation:
  - Fail fast on missing policy markers in CI contract tests.
  - Keep marker vocabulary deterministic and versioned in docs.
  - Keep changes scoped to archive policy marker governance only.

## Interface Contract
- No protocol/wire-format changes without explicit approval.
- Archive policy marker contract documented in:
  - `docs/planning/spec-archive-policy.md`
- Milestone governance marker linkage:
  - `spec_archive_policy_doc=docs/planning/spec-archive-policy.md`

## ADR
- No ADR required (no dependency/protocol/architecture boundary change).
