# Issue #5179 Spec

- Title: Task: implement R42 immediate fixes (signer lock poisoning, ignored-test debt, PRD relocation, draft-spec review)
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Problem Statement
R42 identified immediate stability and hygiene gaps that can be closed in one bounded task: signer test lock poisoning cascade risk, residual ignored-test debt, PRD source file placement drift, and stale draft-spec review debt.

## Scope
In:
- Harden signer env-lock usage so poisoned lock state does not cascade failures.
- Remove `#[ignore]` debt from deep-lane mutation coverage test by using explicit local-heavy env gating.
- Refresh ignored-test inventory fixtures and metadata to match post-change inventory.
- Relocate PRD source file from repo root into docs tree and update all known references.
- Restore missing notifications-consumer lane wrapper symlink required by CI manifest parity checks.
- Review stale R26.4 draft specs (#3881/#3884/#3885/#3886/#3887) and mark as reviewed with staleness notes.
- Preserve shell-surface neutrality.

Out:
- observability endpoint decomposition (tracked separately in #5180)
- doc-contract consolidation (tracked separately in #5184)
- long-horizon API/test-surface audits (tracked separately in #5181)

## Acceptance Criteria
- AC-1: `main_tests::signer_tests` acquires env lock through poison-tolerant handling (`into_inner`) to avoid failure cascades.
- AC-2: `performance_input_mutation_coverage_guided_deep_lane_stress` no longer uses `#[ignore]` and is explicitly local-heavy gated by env.
- AC-3: PRD markdown source is moved under `docs/planning/` and all in-repo references to old root path are updated.
- AC-4: Specs #3881/#3884/#3885/#3886/#3887 move from Draft to Reviewed with explicit 2026-02-19 staleness-review notes.
- AC-5: Shell-surface ratio target status is `improved|neutral` (no net shell LOC increase).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | `cargo test -p kamn-node main_tests::signer_tests -- --nocapture` | Signer tests pass and lock poisoning cascade path is removed |
| C-02 | AC-2 | Functional | `cargo test -p kamn-core --test input_mutation_coverage_guided -- --nocapture` | Deep-lane test is env-gated without `#[ignore]` |
| C-03 | AC-3 | Conformance | `rg -n \"kamn-data-layer-prd\\.docx\\.md\" specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md docs/plans/2026-02-18-kamn-data-layer-prd-execution-plan.md -S` | Only docs-scoped PRD path remains in tracked references |
| C-04 | AC-4 | Regression | `rg -n \"Status: Reviewed|Staleness Review \\(2026-02-19\\)\" specs/3881/spec.md specs/3884/spec.md specs/3885/spec.md specs/3886/spec.md specs/3887/spec.md -S` | All five flagged specs are reviewed and annotated |
| C-05 | AC-5 | Governance | `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` | Fast-mode CI tool regression suite passes with updated ignored-test fixtures and wrapper parity |

## Test Mapping
- C-01 -> `main_tests::signer_tests` full suite
- C-02 -> `input_mutation_coverage_guided` integration/functional suite
- C-03 -> PRD reference grep verification
- C-04 -> stale-spec marker grep verification
- C-05 -> shell-surface neutrality check in diff summary

## Success Metrics
- Immediate R42 high/low findings covered by this task are closed with deterministic verification evidence.
