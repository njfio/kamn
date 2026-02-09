# Kolme Integration Roadmap and Version Compatibility Guards (Issue #780)

This roadmap tracks the compatibility policy used to keep KAMN reproducible
across Kolme upgrades.

## Scope

- Fast lane:
  - validate declared KAMN/Kolme version pair compatibility.
  - run bounded replay smoke over deterministic fixture subset.
- Scheduled deep lane:
  - run full version replay fixture matrix.
  - emit machine-readable replay report for audit evidence.

## Validator Contract

- Validator command:
  - `python3 scripts/kolme/validate_version_compatibility.py --kamn-version 1.1.0 --kolme-release-tag v0.15.2 --ci-fast-gate PASS --output-json /tmp/kolme-version-report.json`
- Replay matrix command:
  - `python3 scripts/kolme/run_version_compatibility_replay.py --fixture fixtures/kolme_compatibility/version_compatibility_cases.json --output-json /tmp/kolme-version-replay-report.json`
- Runtime commit contract lane:
  - `bash scripts/kolme/run_runtime_commit_contract_lane.sh`
  - fixture: `fixtures/kolme_commit/runtime_commit_request_cases.txt`
- Fast contract lane:
  - `bash scripts/kolme/run_version_compatibility_contract_lane.sh`
- Scheduled deep lane:
  - `bash scripts/kolme/run_version_compatibility_replay_deep_lane.sh --output-json kolme-version-compatibility-report.json`

## Runtime and Cost Policy

- Fast lane budget:
  - `run_version_compatibility_contract_lane.sh` enforces a hard budget of 60 seconds.
- PR safety:
  - replay smoke uses `--max-cases 2` via contract lane to keep cost low.
- Scheduled-only work:
  - full replay matrix and artifact publication run in deep workflow only.

## Ownership and Rollout

- Backend owner:
  - maintains validator rule set and fixture updates.
- QA owner:
  - tracks replay fixture growth and ensures runtime remains bounded.
- Release owner:
  - requires replay artifact in go/no-go evidence review.

## Regression Guard

- Known incompatible signature (`kamn 1.2.x` + `kolme v0.14.x`) remains blocked (`Regression: #775`).
- Malformed runtime commit request shapes remain fail-closed (`Regression: #825`).

## Local Validation

```bash
bash scripts/kolme/test_validate_version_compatibility.sh
bash scripts/kolme/test_run_version_compatibility_contract_lane.sh
bash scripts/ci/test_select_targets.sh
bash scripts/ci/test_workflow_scope_policy.sh
```
