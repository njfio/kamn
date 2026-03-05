# Spec: Issue #6378 - Enforce Phase 6 integration evidence before spec-backed closure

## Objective

Prevent issue/spec closure without explicit Phase 6 integration evidence by adding a deterministic fail-closed policy checker and contributor workflow guidance.

## Inputs/Outputs

- Inputs:
  - top-level issue specs under `specs/*.md`
  - contributor closure workflow in `.github/CONTRIBUTING.md`
  - CI tooling regression suite in `scripts/ci/test_ci_tools.sh`
- Outputs:
  - deterministic policy checker for closure-ready specs
  - contract tests validating pass/fail drift behavior
  - contributor-facing remediation guidance for failed checks

## Boundaries/Non-goals

- In scope:
  - policy checker + contract tests for Phase 6 evidence requirements on closure-ready specs.
  - CI tooling lane integration and contributor docs updates.
- Out of scope:
  - changing runtime behavior.
  - retrofitting historical specs not marked closure-ready.
  - changing GitHub issue template structure.

## Failure modes

- FM-1: closure-ready spec is missing Phase 6 section and still passes.
- FM-2: checker fails without deterministic reason markers, reducing triage quality.
- FM-3: checker/report wiring drifts and CI no longer enforces the rule.
- FM-4: docs omit remediation guidance, causing contributor confusion.

## Acceptance criteria (testable booleans)

- [ ] AC-1: policy checker fails closed when a closure-ready spec (`Status: Implemented`) lacks `## Phase 6 integration evidence`.
- [ ] AC-2: policy checker fails closed when a closure-ready spec has Phase 6 heading but lacks explicit execution evidence markers.
- [ ] AC-3: policy checker emits deterministic markers (`status`, `final_decision`, `reason_taxonomy_version`, `reason_codes`) and a JSON report.
- [ ] AC-4: CI tool regression lane executes the new Phase 6 policy contract test.
- [ ] AC-5: contributor docs include the checker command and remediation steps before issue closure.

## Files to touch

- `specs/6378-phase6-evidence-enforcement.md`
- `scripts/ci/check_spec_phase6_evidence_policy.sh` (new)
- `scripts/ci/test_check_spec_phase6_evidence_policy.sh` (new)
- `scripts/ci/test_ci_tools.sh`
- `.github/CONTRIBUTING.md`

## Error semantics

- Checker must fail closed (`status=fail`, non-zero exit) on missing required evidence markers for closure-ready specs.
- Checker must preserve deterministic reason taxonomy and machine-readable reason codes.
- No silent fallback behavior.

## Test plan

- RED:
  - add contract tests expecting deterministic failure for closure-ready specs missing Phase 6 evidence markers.
  - verify contract lane fails before checker implementation.
- GREEN:
  - implement checker and produce deterministic marker/JSON output.
  - integrate contract test into `scripts/ci/test_ci_tools.sh` fast mode.
- REFACTOR:
  - keep checker helpers small and readable with stable reason taxonomy constants.
- INTEGRATION:
  - run checker contract test directly and through CI-tools fast-mode lane.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
