# Spec — Issue #4824

- Title: Subtask: replace CI wave budget trend duplicate scripts with parameterized checker
- Parent: Parent task: #4813
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Consolidate duplicated non-Kolme wave1-wave19 wrapper-family budget trend checker scripts into a single parameterized runner while preserving existing checker entrypoint names and CI contracts.

## Problem Statement

Nineteen near-identical checker scripts differ only by wave id and threshold fixture path (plus a wave-19 runtime budget flag). This duplicated logic increases maintenance overhead and drift risk.

## Scope

In scope:
- add shared checker runner: `scripts/ci/check_non_kolme_wave_wrapper_family_budget_trend_impl.sh`
- convert `check_non_kolme_wave1..19_wrapper_family_budget_trend.sh` files to symlink entrypoints
- preserve wave-19 runtime budget behavior (`KAMN_NON_KOLME_WAVE19_TREND_MAX_SECONDS`)
- add explicit topology contract test for shared runner + wave symlink entrypoints

Out of scope:
- wave wrapper-matrix script consolidation in framework (`#4823`, already complete)
- test harness and JSON helper migrations (`#4825`, `#4826`)
- CI workflow definition changes

## Acceptance Criteria

- AC-1: A single shared runner resolves wave id and executes trend checks for wave1-wave19 with the correct threshold fixture.
- AC-2: Existing checker entrypoint names remain callable and pass existing wave budget trend test contracts.
- AC-3: Duplicate checker script bodies are removed and replaced with parameterized symlink entrypoints without CI regression.

## Conformance Cases

- C-01 (AC-1): `scripts/ci/test_non_kolme_wave_budget_trend_runner_contract.sh` fails when shared runner/symlink topology is missing and passes when topology is correct.
- C-02 (AC-2): `for wave in {1..19}; do bash scripts/ci/test_check_non_kolme_wave${wave}_wrapper_family_budget_trend.sh; done` passes with unchanged entrypoint names.
- C-03 (AC-3): `bash scripts/ci/test_ci_tools.sh` passes after consolidation with non-Kolme wave budget trend checks still integrated.

## Success Metrics / Signals

- 19 duplicated checker scripts converted to symlink entrypoints.
- Shared checker logic centralized in one implementation file (`76` lines) plus one contract test (`34` lines), replacing `197` duplicated checker lines.
- Full CI tools regression remains green.
