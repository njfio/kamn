# 6546 Enforce 50-Commit Capability Moratorium

## Objective
Upgrade the existing governance/feature commit-ratio CI gate so it enforces a temporary capability-first moratorium: the evaluated rolling window must contain at least 80% feature/capability commits across the latest 50 non-merge commits. Reuse the current checker and report path instead of introducing a second governance-policy mechanism.

## Inputs/Outputs
### Inputs
- newline-delimited non-merge commit subjects captured by CI
- configured rolling window size
- configured minimum feature ratio / maximum governance ratio
- output JSON path for the checker report

### Outputs
- fail-closed stdout markers for status, ratios, and window counts
- schema-versioned JSON report consumed by CI artifacts
- fast-gate workflow wiring that evaluates the rolling moratorium deterministically on pull requests
- contributor/CI documentation describing the moratorium semantics

## Boundaries/Non-goals
- Do not rewrite historical review documents or reclassify old commits.
- Do not add a new standalone governance budget or review-only marker family.
- Do not change unrelated CI gates.
- Do not broaden commit-type classification beyond deterministic handling required for the current checker.

## Failure Modes
- commit subject file is missing or empty
- commit subject prefix is unclassified
- rolling window contains fewer than the required feature ratio
- invalid checker arguments are supplied
- workflow wiring drifts from the documented 50-commit / 80% feature configuration

## Acceptance Criteria
- AC-1: The checker can evaluate only the latest N non-merge commit subjects from a supplied subject list.
- AC-2: The checker fails closed when the evaluated 50-commit window contains feature ratio `< 0.80` or governance ratio `> 0.20`.
- AC-3: The checker still fails closed on unclassified commit prefixes and empty input.
- AC-4: Fast Gate invokes the existing checker with the rolling-window moratorium configuration rather than the old PR-local 50/50 threshold.
- AC-5: CI/contributor docs describe the moratorium as a temporary capability-first control on the latest 50 non-merge commits.

## Files To Touch
- `scripts/ci/check_governance_feature_commit_ratio.py`
- `scripts/ci/test_check_governance_feature_commit_ratio.sh`
- `.github/workflows/ci-fast-gate.yml`
- docs that describe the gate, likely `docs/ci/strategy.md` and/or `.github/CONTRIBUTING.md`
- any existing docs contract test that pins the CI strategy markers

## Error Semantics
- The checker remains fail-closed: any invalid input, empty subject window, unknown commit prefix, or threshold breach returns exit code `1` and emits deterministic reason codes.
- CI wiring must not silently fall back to PR-local-only evaluation or a weaker threshold.
- Documentation drift is surfaced through deterministic contract or regression tests rather than advisory comments.

## Test Plan
- C-01 Functional/Red: checker test fixture proves a passing 50-subject window at exactly 80% feature / 20% governance.
- C-02 Functional/Red: checker test fixture proves failure when the evaluated rolling window drops below 80% feature.
- C-03 Regression/Red: checker test fixture proves only the latest configured window is counted, not the full subject list.
- C-04 Integration/Red: workflow wiring regression test proves Fast Gate passes the rolling-window flags and moratorium thresholds.
- C-05 Integration/Green: targeted doc/contract test proves CI strategy docs describe the 50-commit capability moratorium.
