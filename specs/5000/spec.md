# Issue #5000 Spec

- Title: Task: execute docs-contract wrapper remediation and second spec-archive wave
- Status: Implemented
- Type: task
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Deletion-wave cleanup removed superseded shell wrappers, but `docs/ci/strategy.md` and docs-contract assertions still referenced several deleted wrapper commands, producing deterministic docs test failures on `main`. The archive pipeline is implemented but requires a second migration wave to move remaining implemented specs into `specs/archive/` and keep active-tree surface maintainable.

## Acceptance Criteria
- AC-1: `docs/ci/strategy.md` references deleted wrapper entrypoints only via manifest-runner equivalents; docs contract tests pass.
- AC-2: All currently implemented-but-unarchived issue spec directories are migrated by archive wave 2, with active-tree placeholders updated to `ARCHIVED.md`.
- AC-3: Archive-policy enforcement checks pass after wave-2 migration with deterministic status markers.
- AC-4: Issue/process/spec lifecycle artifacts for #5000 are synchronized to `Implemented`.

## Scope
In scope:
- `docs/ci/strategy.md` wrapper-reference remediation.
- `crates/kamn-core/tests/ci_strategy_docs.rs` assertion updates to manifest-runner command surface.
- Archive-wave execution for implemented specs and archive index/pointer updates.
- Policy/checker validation for docs + archive contracts.

Out of scope:
- New shell governance framework features.
- Net-new protocol/runtime behavior changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture` | all docs-contract assertions pass with no deleted-wrapper references |
| C-02 | AC-2 | Functional | run archive wave-2 tool over repo with detected candidate ids | 44 implemented issue specs archived; active paths contain `ARCHIVED.md` pointers |
| C-03 | AC-3 | Integration/Regression | `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <tmp>` + contract tests | checker returns `status=ok` and deterministic counters |
| C-04 | AC-4 | Functional | inspect `specs/5000/*` and issue metadata | lifecycle markers and status fields are aligned to implemented completion |

## Test Mapping
- AC-1:
  - `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture`
- AC-2:
  - `python3 scripts/ci/archive_completed_specs.py --repo-root .`
- AC-3:
  - `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json /tmp/spec-archive-wave2.json`
  - `bash scripts/ci/test_check_spec_archive_policy.sh`
- AC-4:
  - `specs/5000/spec.md`
  - `specs/5000/plan.md`
  - `specs/5000/tasks.md`

## Success Metrics
- `ci_strategy_docs` suite returns zero failures.
- Archive-wave backlog for implemented-but-unarchived issue specs is reduced to zero for current candidate set (post-wave count `0` under accepted status patterns).
- Archive policy checker remains fail-closed and green on repository state after migration.
