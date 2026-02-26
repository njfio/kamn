# Plan: Issue #6051

## Approach
1. Implement `scripts/ci/check_review_document_freeze.py`:
   - parse freeze manifest marker `review_document_freeze_entries_csv`,
   - load changed-file list from input,
   - fail closed on invalid/missing manifest and on frozen-file modifications,
   - emit deterministic JSON + stdout key-value markers.
2. Add RED-first script test harness:
   - pass case with non-frozen changes,
   - failure when frozen file appears in changed set,
   - failure on missing/invalid manifest.
3. Wire checker into fast gate:
   - pull-request step to capture changed files and run checker,
   - upload JSON artifact.
4. Add contract drift enforcement:
   - include checker test in `scripts/ci/test_ci_tools.sh` fast mode,
   - include command in `scripts/ci/test_ci_tools_command_surface_contract.sh`,
   - include workflow assertions in `scripts/ci/test_workflow_scope_policy.sh`.
5. Update docs/contract parity:
   - add strategy markers in `docs/ci/strategy.md`,
   - add docs assertion in `crates/kamn-core/tests/ci_strategy_docs.rs`.
6. Run targeted verification commands and collect RED/GREEN evidence.

## Affected Modules
- `scripts/ci/check_review_document_freeze.py` (new)
- `scripts/ci/test_check_review_document_freeze.sh` (new)
- `.github/workflows/ci-fast-gate.yml`
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_ci_tools_command_surface_contract.sh`
- `scripts/ci/test_workflow_scope_policy.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks / Mitigations
- Risk: false positives due to path-normalization mismatch.
  Mitigation: normalize paths to repo-relative POSIX form and compare exact manifest entries.
- Risk: workflow bypass through selector conditions.
  Mitigation: run checker on all pull requests independent of `run_ci_tool_checks`/`docs_only`.
- Risk: manifest format drift.
  Mitigation: explicit schema/marker validation and fail-closed reason codes.

## Interfaces / Contracts
- Checker command:
  - `python3 scripts/ci/check_review_document_freeze.py --changed-files-file <path> --freeze-manifest docs/review/review-document-freeze.manifest --output-json <path>`
- Report schema:
  - `kamn.ci.review-document-freeze-gate-report.v1`.
- Reason taxonomy:
  - `kamn.ci.review-document-freeze-gate-reason-taxonomy.v1`.
