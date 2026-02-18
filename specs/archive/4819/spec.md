# Spec — Issue #4819

- Title: Subtask: implement scripts/lib/common.sh primitives and complete pilot migration wave
- Parent: Parent task: #4811
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Deliver `scripts/lib/common.sh` primitives and migrate the pilot scripts/dispatcher slice with deterministic tests and fail-closed governance behavior.

## Problem Statement

Without a shared common shell library and validated pilot migration, the broader shell-surface reduction program cannot safely progress with measurable, reversible increments.

## Scope

In scope:
- add `scripts/lib/common.sh` with shared helpers (`assert_eq`, `assert_contains`, `extract_value`, `emit_fallback_error`, `KAMN_ROOT`)
- migrate pilot scripts to source `common.sh` and replace local duplicated helper implementations
- migrate non-Kolme dispatcher fallback helper/root resolution to `common.sh`
- failing-to-passing contract tests for the changed surface
- spec/docs updates for changed behavior

Out of scope:
- phase work outside this subtask boundary
- unrelated refactors

## Acceptance Criteria

- AC-1: `scripts/lib/common.sh` provides stable shared primitives consumed by the pilot migration set.
- AC-2: Pilot-migrated scripts preserve behavior and deterministic contract outputs.
- AC-3: Pilot migration reduces duplicated helper/ROOT_DIR boilerplate in touched scripts.
- AC-4: Red/green regression evidence is captured in issue/PR artifacts.

## Conformance Cases

- C-01 (AC-1): `bash scripts/framework/test_common_shell_library.sh` passes and validates the shared helper contract.
- C-02 (AC-2): `bash scripts/deploy/test_generate_dr_evidence_bundle.sh`, `bash scripts/deploy/test_generate_staging_rehearsal_bundle.sh`, and `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` pass unchanged behavior checks.
- C-03 (AC-2): `bash scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh` passes after dispatcher migration to `common.sh`.
- C-04 (AC-3): touched scripts remove local duplicate helper implementations and local `ROOT_DIR` computation in favor of sourced `KAMN_ROOT`.
- C-05 (AC-4): RED evidence captured from initial failure before library creation; GREEN evidence captured by passing suite in this change.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Touched scripts show measurable local duplication reduction by consolidating helper definitions into `common.sh`.
