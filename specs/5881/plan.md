# Plan: Issue #5881 - Shell LOC Reduction Wave (No Behavior Change)

- Issue: #5881
- Spec: `specs/5881/spec.md`
- Last Updated: 2026-02-24

## Approach
1. Record pre-change shell metric baseline.
2. Apply non-functional whitespace reduction to one high-volume shell script.
3. Run selector regression and shell/docs contract checks.
4. Record post-change shell metric and confirm net reduction.

## Affected Modules
- `scripts/ci/select_targets.sh`
- `specs/5881/*`

## Risks / Mitigations
- Risk: accidental semantic changes in shell control flow.
- Mitigation: perform whitespace-only edits and run full selector regression script.

## Interfaces / Contracts
- No interface/output key changes are allowed for selector outputs.
- Shell LOC contract outputs remain unchanged schema.

## ADR
No ADR required.
