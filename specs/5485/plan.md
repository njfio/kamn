# Issue #5485 Plan - Refresh R49 Artifact Baseline Markers

## Approach
1. Capture current deterministic values for open issues/milestones/remote heads and ignored-test drift status.
2. Update `docs/review/gaps-and-issues-r49.md` by appending a post-publication revalidation section + markers and outputs.
3. Run verification checks (ignored-test drift, fmt, strict clippy, targeted docs contract tests).

## Affected Modules
- `docs/review/gaps-and-issues-r49.md`
- `specs/milestones/r50-8-r49-review-artifact-post-publication-baseline-revalidation/index.md`
- `specs/5485/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: accidental overwrite of publication-time snapshot context.
  - Mitigation: append separate revalidation section and keep original snapshot intact.
- Risk: marker drift between command run and committed content.
  - Mitigation: run commands immediately before edit and embed outputs verbatim.

## Interfaces / Contracts
- Documentation artifact contract only; no code/API/protocol change.

## Validation Strategy
- `gh issue list --repo njfio/kamn --state open --limit 200`
- `gh api repos/njfio/kamn/milestones?state=open --paginate --jq 'length'`
- `git ls-remote --heads origin | wc -l`
- `bash scripts/ci/check_ignored_test_inventory_drift.sh --baseline-file fixtures/ci/ignored_test_inventory_baseline.json --metadata-file fixtures/ci/ignored_test_inventory_metadata.json --promotion-criteria-file fixtures/ci/ignored_test_promotion_criteria.json --output-json /tmp/ignored-test-inventory-drift-report-r49-revalidate.json`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features --manifest-path Cargo.toml -- -D warnings`
