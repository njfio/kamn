# Spec — Issue #4814

- Title: Task: deploy shared test harness and JSON helper utilities across shell contracts
- Parent: Parent story: #4808
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Implement phases 4-5 by extracting reusable test/JSON boilerplate helpers and migrating high-duplication scripts.

## Problem Statement

Current script surface includes large duplicated boilerplate and uneven governance boundaries that increase maintenance burden.

## Scope

In scope:
- phase-aligned implementation and regression checks
- deterministic reason-taxonomy and compatibility markers where applicable
- bounded CI/runtime governance requirements

Out of scope:
- unrelated runtime feature delivery
- non-deterministic policy behavior

## Acceptance Criteria

- AC-1: test_harness.sh supports deterministic reusable assertions/setup patterns.
- AC-2: JSON helper adoption removes repeated inline JSON construction patterns.
- AC-3: Migration preserves contract lane pass/fail semantics.

## Conformance Cases

- C-01 (AC-1): `bash scripts/lib/test_test_harness_migration_contract.sh` passes (subtask `#4825`, PR `#4842`).
- C-02 (AC-2): `bash scripts/lib/test_json_write_helper_migration_contract.sh` passes with helper adoption and zero legacy ROOT_DIR cat JSON heredoc writers (subtask `#4826`, PR `#4843`).
- C-03 (AC-3): `bash scripts/ci/test_ci_tools.sh` passes after both subtask migrations.

## Success Metrics / Signals

- Shared test harness landed and adopted by migrated wrapper-family test flows.
- Shared JSON write helper command landed and adopted across 89 scripts (168 write sites).
- Full CI tool regression suite remains green post-migration.
