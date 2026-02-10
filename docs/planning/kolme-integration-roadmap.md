# Kolme Integration Roadmap and Version Compatibility Guards (Issues #780, #1401)

This roadmap tracks the compatibility policy used to keep KAMN reproducible
across Kolme upgrades.

## Scope

- Fast lane:
  - validate declared KAMN/Kolme version pair compatibility.
  - validate declared upstream/fork tuple compatibility for `fpco/kolme` and `njfio/kolme_fork`.
  - run bounded replay smoke over deterministic fixture subset.
- Scheduled deep lane:
  - run full version replay fixture matrix.
  - emit machine-readable replay report for audit evidence.

## Validator Contract

- Validator command:
  - `python3 scripts/kolme/validate_version_compatibility.py --kamn-version 1.1.0 --kolme-release-tag v0.15.2 --ci-fast-gate PASS --output-json /tmp/kolme-version-report.json`
- Fork compatibility evidence command:
  - `python3 scripts/kolme/generate_fork_compatibility_evidence.py --upstream-release-tag v0.15.2 --fork-release-tag v0.15.2 --fork-repo njfio/kolme_fork --fork-ref refs/heads/main --ci-fast-gate PASS --output-json /tmp/kolme-fork-compatibility-report.json`
  - fixture: `fixtures/kolme_compatibility/fork_compatibility_cases.json`
- Replay matrix command:
  - `python3 scripts/kolme/run_version_compatibility_replay.py --fixture fixtures/kolme_compatibility/version_compatibility_cases.json --output-json /tmp/kolme-version-replay-report.json`
- Runtime commit contract lane:
  - `bash scripts/kolme/run_runtime_commit_contract_lane.sh`
  - fixture: `fixtures/kolme_commit/runtime_commit_request_cases.txt`
- Runtime commit adapter contract reference:
  - `docs/foundation/kolme-runtime-commit-client.md`
- Runtime commit replay policy checker:
  - `python3 scripts/kolme/check_runtime_commit_replay_policy.py --operation-id op-go-001 --idempotency-key kolme-runtime-commit:op-go-001:state:agent:1:12 --receipt-provider kolme-local --expected-receipt-provider kolme-local --receipt-commit-id kolme-commit:op-go-001:agent:1:12 --expected-receipt-commit-id kolme-commit:op-go-001:agent:1:12 --nonce-monotonic true --replay-detected false --payload-hash-match true --receipt-finality FINAL --ci-fast-gate PASS --output-json /tmp/kolme-runtime-commit-replay-policy.json`
- Runtime commit replay matrix command:
  - `python3 scripts/kolme/run_runtime_commit_replay_tamper_matrix.py --fixture fixtures/kolme_commit/runtime_commit_replay_tamper_cases.json --output-json /tmp/kolme-runtime-commit-replay-report.json`
- Runtime commit replay contract lane:
  - `bash scripts/kolme/run_runtime_commit_replay_contract_lane.sh`
  - fixture: `fixtures/kolme_commit/runtime_commit_replay_tamper_cases.json`
- Runtime commit adapter replay/finality contract lane:
  - `bash scripts/kolme/run_runtime_commit_adapter_contract_lane.sh`
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
- Runtime commit finality projection blocks invalid lifecycle regression to pending (`Regression: #826`).
- Runtime commit replay/tamper mismatch policy emits fail-closed reason codes (`Regression: #827`).
- Adapter provider mismatch and non-final receipt handling remain fail-closed (`Regression: #979`).
- Adapter replay/finality reason-code drift remains fail-closed (`Regression: #980`).
- Fork release-tag drift remains fail-closed (`Regression: #1401`).

## Local Validation

```bash
bash scripts/kolme/test_validate_version_compatibility.sh
bash scripts/kolme/test_generate_fork_compatibility_evidence.sh
bash scripts/kolme/test_run_version_compatibility_contract_lane.sh
bash scripts/kolme/test_run_runtime_commit_contract_lane.sh
bash scripts/kolme/test_check_runtime_commit_replay_policy.sh
bash scripts/kolme/test_run_runtime_commit_replay_contract_lane.sh
bash scripts/ci/test_select_targets.sh
bash scripts/ci/test_workflow_scope_policy.sh
```
