# Spec: Issue #5842 - Close R56 Governance/Audit Gaps with Enforceable Contracts

- Issue: #5842
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
R56 audit findings report unresolved structural governance coupling, growing production `expect()` paths, inconsistent spec-dir contamination claims, unenforced post-publication review moratorium behavior, and flat shell-surface reduction signals. These findings need executable enforcement and measurable remediation instead of marker-only declarations.

## Scope
In scope:
- Enforce structural-coupling governance policy using executable review-contract checks.
- Harden production `expect()` inventory counting semantics and reduce panic-style paths in active service/client code.
- Ensure tracked-only spec-dir counting remains implemented and regression-protected.
- Enforce post-publication review immutability with deterministic freeze markers/contracts.
- Deliver measurable shell-surface reduction and updated policy markers.

Out of scope:
- Rewriting merged historical commit history.
- Protocol/wire-format schema changes.
- New dependency introduction.

## Acceptance Criteria
- AC-1: Governance structural-coupling policy has executable enforcement that fails closed when lifecycle-only overhead exceeds target ratio.
- AC-2: Production `expect()` inventory logic is deterministic (no false stripping of non-test cfg blocks), and audit markers reflect actual counted values.
- AC-3: Spec-dir non-regression counting uses tracked-only git semantics with explicit regression coverage against untracked contamination.
- AC-4: Post-publication review docs are immutable under enforced freeze contracts for released review artifacts.
- AC-5: Shell-surface reduction is measurable and recorded with policy-compliant actual delta markers.
- AC-6: Updated docs-contract suites and CI tool regression lanes pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | governance coupling markers above max | fail-closed status |
| C-02 | AC-1 | Regression | lifecycle-only commit coupling regression fixture | detected as over-target |
| C-03 | AC-2 | Unit | cfg attributes containing `"test"` in feature strings | not treated as `cfg(test)` |
| C-04 | AC-2 | Functional | production expect inventory run on workspace | deterministic non-zero counted snapshot + marker consistency |
| C-05 | AC-3 | Regression | create untracked top-level `specs/` dir during test | tracked count unchanged |
| C-06 | AC-4 | Functional | mutate frozen review doc marker/line hash | freeze contract fails closed |
| C-07 | AC-4 | Regression | baseline frozen docs without drift | freeze contract passes |
| C-08 | AC-5 | Integration | shell-surface telemetry/checkers after migration/deletion | improved or policy-valid status |
| C-09 | AC-6 | Conformance | docs-contract + ci-tools suites | green |

## Test Mapping
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract -- --nocapture`
- `bash scripts/ci/test_ci_tools.sh`
- `bash scripts/ci/test_check_no_production_expect.sh`
- `cargo fmt --all --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`

## Success Metrics / Observable Signals
- Governance structural-coupling enforcement path reports fail-closed status when ratio exceeds target.
- Production `expect()` inventory markers are based on corrected deterministic counting.
- Review docs immutability is enforced by deterministic freeze contract checks.
- Shell-surface policy markers report measurable non-regression improvement (or explicit waiver path).
