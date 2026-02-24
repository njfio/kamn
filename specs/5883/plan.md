# Plan: Issue #5883 - Shell LOC Reduction Wave 2 (Selector Test Surface)

- Issue: #5883
- Spec: `specs/5883/spec.md`
- Last Updated: 2026-02-24

## Approach
1. Capture pre-change shell LOC baseline.
2. Apply whitespace-only blank-line removal in `scripts/ci/test_select_targets.sh`.
3. Run selector regression and review docs contract tests.
4. Capture post-change shell LOC and verify net reduction.

## Affected Modules
- `scripts/ci/test_select_targets.sh`
- `specs/5883/*`

## Risks / Mitigations
- Risk: accidental script behavior change.
- Mitigation: whitespace-only edits and full selector test execution.

## Interfaces / Contracts
No output key changes or control-flow changes are allowed.

## ADR
No ADR required.
