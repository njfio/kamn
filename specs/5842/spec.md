# Spec: Issue #5842 - Close R56 Governance/Audit Gaps with Enforceable Contracts

- Issue: #5842
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
R56 audit follow-up identified persistent contract drift: governance structural coupling remained above target, post-publication review immutability was not fail-closed, spec-dir contamination claims required stronger tracked-only semantics, production `expect()` inventory claims required deterministic scoped evidence, and shell-surface reduction lacked enforceable closure evidence.

## Scope
In scope:
- Add executable governance-structural-coupling + post-publication freeze enforcement in review docs-contract tests/policies.
- Correct and enforce crate-freeze attribution markers (S-12/S-13 vs #5831 overstated claim path) through deterministic review markers.
- Harden tracked-only spec-dir counting semantics to explicit git-tree based enumeration with regression coverage.
- Keep production `expect()` inventory deterministic/correctly scoped and enforce non-regression/reduction signals.
- Deliver measurable shell-surface LOC reduction with closure markers and contract validation evidence.

Out of scope:
- Rewriting merged Git history.
- Protocol/wire-format redesign.
- Broad, repo-wide panic-path eradication beyond scoped audit contracts in this issue.

## Acceptance Criteria
- AC-1: Governance structural-coupling status is derived from executable policy checks and fails closed when target ratios are exceeded.
- AC-2: Production `expect()` inventory contract is deterministic, correctly scoped, and reflects reduced/equivalent-no-regression behavior against R55 baseline markers.
- AC-3: Spec-dir contamination prevention uses tracked-only git-tree semantics and proves untracked dir contamination cannot alter counted results.
- AC-4: Post-publication review docs (R51+) are protected by deterministic immutability/freeze checks in docs-contract tests.
- AC-5: R56 marker set includes corrected unresolved-state/attribution evidence for prior overstated claims, and tests validate those markers.
- AC-6: Shell-surface metrics include measurable reduction evidence and status markers with enforced validation.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | R56 governance coupling markers + policy | status computed from ratio and target; fails closed on mismatch |
| C-02 | AC-2 | Functional | production source inventory scan | deterministic count and policy status consistent with markers |
| C-03 | AC-3 | Regression | create untracked top-level `specs/` dir during test | tracked count unchanged |
| C-04 | AC-3 | Unit | tracked spec-dir counter command path | explicit git-tree command semantics |
| C-05 | AC-4 | Regression | mutate frozen review text/hash expectation | docs-contract fails |
| C-06 | AC-5 | Functional | R56 unresolved/attribution marker block | markers present and internally consistent |
| C-07 | AC-6 | Functional | shell LOC/ratio markers and measured delta | reduction evidence parseable and status consistent |

## Test Mapping
- `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`
- `bash scripts/ci/test_check_no_production_expect.sh`
- `bash -n scripts/ci/test_ci_tools.sh`

## Success Metrics / Observable Signals
- Freeze enforcement blocks further silent post-publication review edits.
- Spec-dir contamination regression remains green with explicit tracked-only semantics.
- Governance/attribution markers stop overstating closure and reflect enforceable status.
- Shell LOC delta is negative for touched scripts in this issue.
