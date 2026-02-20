# Issue #4044 Plan

- Issue: #4044
- Milestone: specs/milestones/r27-12-api-schema-evolution-and-compatibility-governance/index.md

## Approach
1. Add `scripts/ci/check_api_compatibility_ci_dry_run_governance.py` to evaluate:
   - #4041/#4042/#4043 dry-run report contract parity,
   - baseline runtime thresholds from `fixtures/ci/`,
   - fast-mode selector and workflow heavy-lane exclusion,
   - strategy/ops docs parity + remediation marker completeness.
2. Add `fixtures/ci/api_compatibility_ci_dry_run_governance_thresholds.env` with deterministic baseline keys.
3. Add a Rust contract suite:
   - `crates/kamn-core/tests/compatibility_ci_dry_run_governance_contract.rs`
   - generate dry-run reports, invoke checker, assert pass/fail and reason-code determinism.
4. Update docs marker sections:
   - `docs/ci/strategy.md`
   - `docs/ops/configuration.md`
5. Wire checker contract into CI tools coverage:
   - add `cargo test -p kamn-core --test compatibility_ci_dry_run_governance_contract` to fast/full blocks in `scripts/ci/test_ci_tools.sh`.
6. Run targeted verification (`fmt`, `clippy`, tests, shell-surface guardrails).

## Affected Files
- `specs/4044/spec.md`
- `specs/4044/plan.md`
- `specs/4044/tasks.md`
- `scripts/ci/check_api_compatibility_ci_dry_run_governance.py`
- `fixtures/ci/api_compatibility_ci_dry_run_governance_thresholds.env`
- `crates/kamn-core/tests/compatibility_ci_dry_run_governance_contract.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `scripts/ci/test_ci_tools.sh`

## Risks and Mitigations
- Risk: checker brittleness to benign prose edits in docs.
  - Mitigation: enforce deterministic marker keys, not prose paragraphs.
- Risk: shell-surface drift from governance additions.
  - Mitigation: keep shell edits limited to CI-tools invocation lines; implement behavior in Python + Rust.
- Risk: false positives from selector parsing.
  - Mitigation: validate exact required/forbidden command markers in fast-mode block and workflow content.

## Interface Contract
- Checker schema:
  - `kamn.ci.api-compatibility-ci-dry-run-governance-report.v1`
- Checker reason taxonomy:
  - `kamn.ci.api-compatibility-ci-dry-run-governance-reason-taxonomy.v1`
- Threshold fixture:
  - `fixtures/ci/api_compatibility_ci_dry_run_governance_thresholds.env`

## ADR
- Not required (policy/checker/docs parity extension only; no dependency or wire-format changes).
