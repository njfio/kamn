# Post-Billing Closeout Checklist (Issues #68 and #70)

Use this checklist once GitHub Actions billing/spending limits are restored.

## Prerequisites
- Actions jobs can start successfully for this repository.
- `gh` CLI authenticated with `repo` scope.
- Local tools: `bash`, `jq`, `unzip`.

## 1) Trigger and Verify CI Workflows
Run manually once to verify jobs execute:

```bash
gh workflow run ci-fast-gate.yml -R njfio/kamn
gh workflow run ci-deep-validate.yml -R njfio/kamn
gh workflow run ci-flaky-registry.yml -R njfio/kamn
gh workflow run ci-flaky-report-comment.yml -R njfio/kamn
gh workflow run ci-flaky-sync-issues.yml -R njfio/kamn
```

Check recent runs:

```bash
gh run list -R njfio/kamn --limit 30
```

Pass criteria:
- Jobs start and complete (not blocked by billing).
- Artifacts are uploaded where expected.

## 2) Gather Budget Telemetry (Issue #68)
Download and summarize budget artifacts:

```bash
bash scripts/ci/download_and_summarize_budget.sh --repo njfio/kamn --lane fast-gate --limit 30
bash scripts/ci/download_and_summarize_budget.sh --repo njfio/kamn --lane deep-validate --limit 30
```

Pass criteria for Stage-1 targets:
- Fast-gate p95 runtime <= 900 seconds.
- Fast-gate p95 runner-minutes <= 25.
- Deep-validate runtime <= 7200 seconds.
- Cache telemetry appears in summaries (`cache_hit` not always `unknown`).

If criteria fail:
1. Adjust cache/parallel strategy in workflows.
2. Re-run and compare summaries.
3. Document deltas in issue #68.

Issue #68 closeout comment template:

```md
**Outcome:** <measured runtime/cache tuning delivered>
**PR:** #<number>
**Follow-up:** <None or next tuning item>
```

## 3) Verify Flaky Policy Automation (Issue #70)
Validate flaky tooling behavior with live runs:
- `ci-flaky-registry`: uploads weekly report artifact.
- `ci-flaky-report-comment`: posts report comment to issue #70.
- `ci-flaky-sync-issues`: applies `flaky-test` label and updates referenced issues.

Check local registry hygiene:

```bash
bash scripts/ci/check_flaky_registry.sh
```

Pass criteria:
- Scheduled/manual runs succeed.
- Report/comment/sync workflows produce expected outputs.
- Flaky entries have owner, tracking issue, and non-expired date.

Issue #70 closeout comment template:

```md
**Outcome:** <bounded retry + quarantine automation verified on hosted runs>
**PR:** #<number>
**Follow-up:** <None or remaining automation hardening>
```

## 4) Final Program Updates
After #68 and #70 closure:
- Post status update on issue #1 summarizing measured outcomes.
- Keep CI budget/trend commands in team runbooks for recurring review.
