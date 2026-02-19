# Issue #5113 Plan

- Issue: #5113
- Status: Implemented

## Approach
1. Build deterministic issue-id list for all active issue dirs with `Status: Implemented` in `spec.md`.
2. Execute archive tool dry-run against this exact list and capture report.
3. Apply archive move with the same list.
4. Validate archive policy checker and shell governance guardrails.
5. Commit archive wave, open PR, merge, and close issue with measured DoD markers.

## Risks and Mitigations
- Risk level: medium
- Risks:
  - New implemented specs landing during wave execution can change candidate set.
  - Incomplete required artifacts for some issue ids can cause partial failures.
- Mitigations:
  - Freeze candidate list to a file and reuse it for dry-run and apply.
  - Validate post-apply with policy checker before commit.

## Interface Contract
- No runtime API changes.
- Filesystem layout migration only under `specs/`.

## ADR
- Not required for operational archival wave.
