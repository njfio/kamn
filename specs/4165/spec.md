# Spec — Issue #4165

- Title: Subtask: add red tests for env-secret decode buffer zeroization after signer initialization
- Parent: Parent task: #4161
- Milestone: R27.20 Secret material zeroization and signer-rotation governance
- Status: Reviewed
- Priority: P1

## Objective

Add deterministic red/green tests that prove signer env-secret buffers are zeroized when strict signer-source contracts reject a selected path.

## Problem Statement

Strict signer-source precedence checks can fail after reading secret material from environment variables. Without explicit zeroization on that error path, secrets can persist in transient buffers.

## Scope

In scope:
- signer secret precedence failure tests validating zeroization behavior
- regression marker tests for required zeroization code paths

Out of scope:
- hardware-backed key storage
- signer backend protocol redesign

## Acceptance Criteria

- AC-1: A failing-first test exists for strict signer-source precedence failures proving env-secret buffers are scrubbed.
- AC-2: Regression tests fail closed if explicit zeroization markers drift from signer secret ingestion paths.
- AC-3: Tests preserve deterministic signer precedence reason code behavior.

## Conformance Cases

- C-01 (AC-1, Unit): `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact`
- C-02 (AC-2, Regression): `cargo test -p kamn-node regression_signer_secret_source_precedence_path_requires_zeroize_markers -- --exact`
- C-03 (AC-3, Functional): `cargo test -p kamn-node signer::tests::regression_strict_signer_secret_source_precedence_rejects_dual_private_key_envs -- --exact`

## Success Metrics / Signals

- Zeroization contract tests are deterministic and non-flaky.
- Secret-source precedence failures continue emitting `signer_secret_source_precedence_violation`.

