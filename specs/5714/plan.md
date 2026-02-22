# Plan: #5714 Execute R52 Spec-Volume Remediation Tranche-1 (14-Dir Reduction)

## Approach
1. Extend existing spec-volume docs-contract test coverage with new R52 tranche marker requirements (RED first).
2. Capture pre-tranche top-level `specs/` directory count.
3. Delete selected 14 archived issue pairs (`specs/<id>` pointer + `specs/archive/<id>` payload).
4. Update `specs/archive/index.md`:
   - remove corresponding rows
   - update `archived_issue_count`
5. Update review marker docs:
   - R50 non-regression spec-volume baseline/max values
   - R52 post-publication tranche markers and evidence commands
   - README marker template for tranche schema
6. Run verification gates and targeted regression suites.

## Candidate Tranche Set (14 issue IDs)
`4195, 4196, 4197, 4221, 4222, 4223, 4225, 4226, 4227, 4228, 4229, 4230, 4236, 4240`

## Affected Modules
- `specs/<id>/` (selected 14 directories)
- `specs/archive/<id>/` (selected 14 directories)
- `specs/archive/index.md`
- `docs/review/README.md`
- `docs/review/gaps-and-issues-r50.md`
- `docs/review/gaps-and-issues-r52.md`
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`

## Risks and Mitigations
- Risk: accidental archive-index inconsistency.
  - Mitigation: deterministic id list, row deletion script, archive policy checker run.
- Risk: docs-contract ratchet drift failures.
  - Mitigation: targeted RED/GREEN loops with spec-volume test suite.
- Risk: unintentional deletion outside tranche IDs.
  - Mitigation: explicit loop over fixed numeric list and pre/post existence checks.

## Interfaces / Contracts
- Archive policy checker contract (`scripts/ci/check_spec_archive_policy.sh`).
- Spec-volume ratchet contract (`review_r50_spec_volume_remediation_docs_contract.rs`).
- Review marker template contract (`docs/review/README.md`).

## ADR
No ADR required (governance/docs artifact reduction; no architecture/dependency/protocol change).
