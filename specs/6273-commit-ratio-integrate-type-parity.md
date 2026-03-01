# Issue 6273 - Align commit-ratio checker with AGENTS integrate commit type

## Objective
Ensure the governance/feature commit-ratio checker classifies `integrate(...)` commit subjects as feature activity so CI behavior matches AGENTS-defined commit taxonomy.

## Inputs/Outputs
- Inputs:
  - PR commit subject file consumed by `scripts/ci/check_governance_feature_commit_ratio.py`.
  - Conventional commit subjects including `integrate(<issue>): ...`.
- Outputs:
  - Deterministic report classifying `integrate` as feature type.
  - Updated contract tests proving `integrate` is accepted and counted as feature.

## Boundaries/Non-goals
- In scope:
  - `scripts/ci/check_governance_feature_commit_ratio.py`
  - `scripts/ci/test_check_governance_feature_commit_ratio.sh`
- Out of scope:
  - Changing governance-ratio thresholds.
  - Broader commit taxonomy redesign beyond AGENTS parity.
  - CI workflow topology changes.

## Failure modes
- FM1: `integrate` commit subjects are marked unclassified.
- FM2: report omits `integrate` from `feature_commit_types_csv`.
- FM3: existing known-type behavior regresses when adding `integrate`.

## Acceptance criteria (testable booleans)
- AC1: checker classifies `integrate(...)` as feature activity.
- AC2: `feature_commit_types_csv` includes `integrate`.
- AC3: existing fixtures for pass/fail ratio behavior remain unchanged.
- AC4: checker contract tests include an explicit `integrate` fixture path.

## Files to touch
- `scripts/ci/check_governance_feature_commit_ratio.py`
- `scripts/ci/test_check_governance_feature_commit_ratio.sh`

## Error semantics
- Unknown commit types still fail closed with `governance_commit_subject_unclassified`.
- `integrate` is no longer treated as unknown.
- Ratio-threshold behavior and reason codes stay unchanged.

## Test plan
- RED:
  - add fixture with `integrate(...)` and assert no unknown-classification violation.
  - verify this fails before implementation.
- GREEN:
  - update feature type set to include `integrate`.
  - rerun checker contract tests.
- REFACTOR:
  - keep taxonomy declarations centralized and deterministic.
- INTEGRATION:
  - run CI tools test lane that includes checker contract test.
