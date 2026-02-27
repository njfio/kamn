# Spec: Issue #6051 - Add fail-closed review-document freeze checker and fast-gate wiring

- Issue: #6051
- Status: Implemented
- Type: task
- Priority: P1
- Area: governance
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #6050

## Problem Statement
Review-document freeze and post-publication moratorium policies are currently enforced primarily by docs-contract test suites. Docs-only PRs can skip those suites and still modify frozen `docs/review/gaps-and-issues-r*.md` artifacts. A direct PR fast-gate checker is required to fail closed on frozen review-document edits.

## Scope
In scope:
- Add deterministic checker for changed-file diff against `docs/review/review-document-freeze.manifest`.
- Fail closed when any changed path matches a frozen review-document entry.
- Fail closed when freeze manifest is missing or malformed.
- Wire checker into `.github/workflows/ci-fast-gate.yml` in pull-request runs independent of docs-only scope.
- Add script/workflow/docs contract tests and documentation markers.

Out of scope:
- Rewriting historical frozen review documents.
- Changing freeze-manifest schema or moratorium policy semantics.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Fast gate runs review-freeze checker on pull requests.
- AC-2: Checker fails closed when a changed file is listed in freeze manifest.
- AC-3: Checker fails closed when freeze manifest is missing or invalid.
- AC-4: Checker emits schema-versioned JSON report with deterministic reason taxonomy.
- AC-5: Command/workflow/docs contract tests enforce checker wiring and marker parity.

## Conformance Cases
- C-01 (Conformance, AC-1): `ci-fast-gate` includes dedicated review-document freeze step.
- C-02 (Functional, AC-2): fixture changed-files list without frozen docs passes.
- C-03 (Regression, AC-2): fixture changed-files list containing frozen docs fails with deterministic reason code.
- C-04 (Regression, AC-3): missing/invalid manifest fails with deterministic reason code.
- C-05 (Conformance, AC-4): checker JSON includes schema version, reason taxonomy version, reason codes, frozen-entry count, and blocked-file list.
- C-06 (Integration, AC-5): CI script command-surface and workflow scope policy tests validate wiring.

## Success Metrics / Observable Signals
- `scripts/ci/test_check_review_document_freeze.sh` passes.
- `scripts/ci/test_ci_tools_command_surface_contract.sh` includes checker test command.
- `scripts/ci/test_workflow_scope_policy.sh` validates workflow step and report artifact.
- PR fast gate produces `ci-review-document-freeze.json` artifact when executed.
