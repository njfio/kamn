# Spec — Issue #4820

- Title: Subtask: execute bulk ROOT_DIR/usage/assert/extract helper migration with compatibility checks
- Parent: Parent task: #4811
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Execute a bulk migration wave that replaces local `ROOT_DIR`/`extract_value`/`assert_eq` boilerplate in evidence-bundle test scripts with shared `scripts/lib/common.sh` primitives.

## Problem Statement

Without a bulk mechanical migration wave, shell duplication remains high and Phase 0 cannot deliver measurable LOC reduction.

## Scope

In scope:
- add migration conformance test for the selected evidence-bundle suite
- migrate selected scripts to source `scripts/lib/common.sh`
- remove local duplicated `extract_value`/`assert_eq`/`ROOT_DIR` bootstrap in selected scripts
- run deterministic compatibility checks for all migrated scripts
- spec/docs updates for changed behavior

Out of scope:
- phase work outside this subtask boundary
- unrelated refactors

## Acceptance Criteria

- AC-1: The selected evidence-bundle migration wave sources `scripts/lib/common.sh` consistently.
- AC-2: Migrated scripts preserve deterministic behavior and continue passing existing tests.
- AC-3: Selected scripts no longer carry local `ROOT_DIR`/`extract_value`/`assert_eq` duplication.
- AC-4: Red/green migration evidence is captured in issue/PR artifacts.

## Conformance Cases

- C-01 (AC-1/AC-3): `bash scripts/framework/test_common_shell_migration_wave_evidence_bundle.sh` enforces `common.sh` sourcing and absence of legacy duplicate helpers for the migration set.
- C-02 (AC-2): all migrated evidence-bundle tests pass in a deterministic suite run (25 scripts).
- C-03 (AC-4): RED output captured before migration (97 violations), followed by GREEN output after migration.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Selected migration wave shows measurable duplicate-helper removal across 25 scripts.
