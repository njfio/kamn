# Plan: Issue #6048

## Approach
1. Implement a standalone fail-closed checker script:
   - Input: newline-delimited commit subjects.
   - Classification: governance vs feature by deterministic prefix map.
   - Output: schema-versioned JSON report.
   - Exit status: non-zero on threshold breach or unknown classification.
2. Add a shell test harness covering:
   - pass case (`governance_ratio <= 0.50`),
   - fail case (`governance_ratio > 0.50`),
   - fail-closed unknown prefix case,
   - JSON contract fields.
3. Wire checker into CI:
   - add to `scripts/ci/test_ci_tools.sh` fast mode,
   - add workflow step in `.github/workflows/ci-fast-gate.yml`,
   - upload report artifact in workflow.
4. Extend workflow command-surface policy tests to assert presence and correctness.
5. Update `docs/ci/strategy.md` and doc-contract test assertions.
6. Run targeted verification commands and collect RED/GREEN evidence for PR.

## Affected Modules
- `scripts/ci/check_governance_feature_commit_ratio.py` (new)
- `scripts/ci/test_check_governance_feature_commit_ratio.sh` (new)
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_ci_tools_command_surface_contract.sh`
- `scripts/ci/test_workflow_scope_policy.sh`
- `.github/workflows/ci-fast-gate.yml`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks / Mitigations
- Risk: false positives from commit-subject variability.
  Mitigation: explicit allowlisted prefixes and fail-closed reason codes for unknowns.
- Risk: workflow drift bypasses enforcement.
  Mitigation: workflow contract test assertions with exact command fragments.
- Risk: ratio gaming by commit-message relabeling.
  Mitigation: keep policy visible in docs and enforce deterministic classifier map.

## Interfaces / Contracts
- Checker command:
  - `python3 scripts/ci/check_governance_feature_commit_ratio.py --commit-subjects-file <path> --max-governance-ratio 0.50 --output-json <path>`
- Workflow command contract:
  - collect `git log --no-merges --pretty=format:%s` for PR base/head range,
  - run checker,
  - upload JSON report artifact.
- Report schema:
  - `schema_version=kamn.ci.governance-feature-commit-ratio-report.v1`.
