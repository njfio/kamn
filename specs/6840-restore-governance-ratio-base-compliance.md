# 6840 Restore Governance Ratio Base Compliance

## Objective
Restore `main` branch compliance for the governance/feature commit-ratio gate without changing the moratorium base SHA, thresholds, or workflow semantics.

## Inputs/Outputs
- Input: current `origin/main` history from `GOVERNANCE_FEATURE_COMMIT_RATIO_MORATORIUM_BASE_SHA=d2c2fe1b901a1d53ea419f31778e1d836f2b1323`, evaluated over the latest 50 non-merge commits.
- Output: a repair branch whose non-merge history is sufficient for the existing checker to return `status=ok` at the current head.

## Boundaries/Non-goals
- Do not change `.ci/governance-feature-commit-ratio-moratorium.env`.
- Do not change workflow thresholds or checker semantics.
- Do not mute or bypass the gate.
- Do not rewrite `main` history.

## Failure modes
- The branch head still returns `status=violation` after the repair train.
- The repair changes the moratorium base SHA or gate threshold.
- The repair relies on governance-only paths and therefore fails to move the feature ratio.
- Evidence is not recorded with the exact checker command and output.

## Acceptance criteria
- [ ] The exact checker command from the issue returns `status=ok` for the repair branch head using the existing moratorium base SHA and `window-size=50`.
- [ ] The repair branch does not modify the moratorium base SHA, threshold, or checker logic.
- [ ] The repair is implemented through compliant non-merge branch history, not CI bypass.
- [ ] Regression coverage exists in a crate test target that shells out to the existing checker.
- [ ] The spec records the exact passing command and final output.

## Files to touch
- `specs/6840-restore-governance-ratio-base-compliance.md`
- `crates/kamn-core/tests/governance_feature_commit_ratio_base_compliance.rs`
- `crates/kamn-core/tests/governance_feature_commit_ratio_base_compliance/`

## Error semantics
- The checker remains the authority and must hard-fail with its existing reason codes when the ratio is above threshold.
- Regression tests must fail loudly if the checker cannot run, the JSON report is missing fields, or the branch head remains out of compliance.

## Test plan
- Add a red crate test that shells out to `scripts/ci/check_governance_feature_commit_ratio.py` against the current branch head and asserts `status=ok`; it must fail before the repair train is complete.
- Add focused regression tests for historical activation-base behavior and report schema invariants under `crates/kamn-core/tests/governance_feature_commit_ratio_base_compliance/`.
- Re-run the exact checker command from the issue against the repair branch head and record the passing output in this spec.

## Final evidence
- Command: `python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha d2c2fe1b901a1d53ea419f31778e1d836f2b1323 --head-sha HEAD --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/6840-final.json`
- status: `violation`
- governance_commit_count: `11`
- feature_commit_count: `39`
- unknown_commit_count: `0`
- governance_ratio: `0.22`
- feature_ratio: `0.78`

