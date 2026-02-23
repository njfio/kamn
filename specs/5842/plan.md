# Plan: Issue #5842 - R56 Governance/Audit Gap Closure

- Issue: #5842
- Spec: `specs/5842/spec.md`
- Status: Implemented

## Approach
1. Implement deterministic review-doc freeze enforcement for released review artifacts.
2. Harden production `expect()` inventory counting by correcting `cfg(test)` detection semantics in Rust/Python checkers.
3. Reconcile and enforce structural-coupling governance ratio policy in review contracts.
4. Confirm tracked-only spec-dir counting remains enforced across affected docs-contract suites.
5. Deliver measurable shell-surface reduction by replacing/removing at least one shell path and updating policy markers.
6. Update R56 review artifact markers/statuses and run full verification gates.

## Affected Modules
- `crates/kamn-core/tests/review_r53_docs_contract.rs`
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `scripts/ci/check_no_production_expect.py`
- `scripts/ci/test_check_no_production_expect.sh`
- `docs/review/gaps-and-issues-r56.md`
- `docs/review/README.md`
- `docs/review/post-publication-moratorium.policy`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- `specs/5842/spec.md`
- `specs/5842/plan.md`
- `specs/5842/tasks.md`

## Interfaces / Contracts
- Review docs freeze contract (deterministic marker/hash baseline).
- Production `expect()` inventory contract and marker formula consistency.
- Governance structural-coupling ratio budget status contract.
- Shell-surface ratio/LOC governance markers (actual-delta reporting).

## Risks and Mitigations
- Risk: Tightening cfg parsing may change inventory counts and fail existing markers.
  - Mitigation: update markers and formulas in same change with deterministic tests.
- Risk: Freeze enforcement could fail due to historical document drift.
  - Mitigation: establish explicit freeze baseline file with deterministic release coverage.
- Risk: Shell-surface deletion can break wrapper/matrix contracts.
  - Mitigation: run full fast ci-tools + targeted wrapper contract tests.

## ADR
- Not required: no new dependencies and no wire-format/protocol changes.
