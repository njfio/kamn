# Issue #5111 Plan

- Issue: #5111
- Status: Implemented

## Approach
1. Identify eligible implemented issue specs with `spec.md`, `plan.md`, and `tasks.md` present.
2. Run archive tool dry-run for the full eligible set.
3. Apply archival move with same deterministic issue-id set.
4. Validate archive policy checker and shell guardrails.
5. Commit archive wave + updated archive index and pointers.

## Risks and Mitigations
- Risk level: medium
- Risks:
  - Missing required files or non-Implemented statuses could fail wave.
  - Index/pointer mismatch could fail policy checker.
- Mitigations:
  - Build deterministic eligible id list before apply.
  - Run dry-run and then apply using same exact list.
  - Run archive policy checker post-apply before commit.

## Interface Contract
- No runtime API changes.
- Filesystem layout migration only under `specs/`.

## ADR
- Not required for operational archival wave.
