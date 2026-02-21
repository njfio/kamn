# Issue #5465 Plan - Ignored-Test Disposition Refresh

## Approach
1. RED: add docs-contract test expecting an R49 re-evaluation artifact + deterministic markers before creating the artifact.
2. Publish `docs/planning/` re-evaluation report with:
   - evidence command markers
   - inventory count marker
   - per-test disposition/rationale table
3. GREEN: rerun new docs-contract test and existing ignored-test drift checker contract.
4. Close with periodic-review process markers.

## Affected Modules
- `docs/planning/2026-02-21-r49-ignored-test-periodic-reevaluation.md` (new)
- `docs/review/gaps-and-issues-r48.md`
- `crates/kamn-core/tests/review_r49_ignored_test_disposition_docs_contract.rs` (new)
- `specs/milestones/r49-1-ignored-test-periodic-reevaluation/index.md`
- `specs/5465/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: inventory drift between live scan and baseline fixture.
  - Mitigation: run deterministic checker command and record outcome markers.
- Risk: missing per-test disposition evidence.
  - Mitigation: require per-test table rows and assert each baseline test name in docs-contract test.

## Interfaces / Contracts
- `ignored_test_disposition_schema_version=kamn.review.ignored-test-disposition.v1`
- `ignored_test_periodic_review_cycle=R49`

## Validation Strategy
- RED:
  - `cargo test -p kamn-core --test review_r49_ignored_test_disposition_docs_contract -- --nocapture`
- GREEN/REGRESSION:
  - `cargo test -p kamn-core --test review_r49_ignored_test_disposition_docs_contract -- --nocapture`
  - `bash scripts/ci/check_ignored_test_inventory_drift.sh --baseline-file fixtures/ci/ignored_test_inventory_baseline.json --metadata-file fixtures/ci/ignored_test_inventory_metadata.json --promotion-criteria-file fixtures/ci/ignored_test_promotion_criteria.json --output-json /tmp/ignored-test-inventory-drift-report-r49.json`
  - `cargo fmt --check`
