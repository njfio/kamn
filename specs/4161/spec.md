# Spec - Issue #4161

- Title: Task: implement key-material zeroization across env decode and signing lifecycle paths
- Parent: #4159
- Milestone: R27.20 Secret material zeroization and signer-rotation governance
- Status: Implemented
- Priority: P1

## Problem Statement

Signer initialization and request paths can retain decoded private-key material in transient buffers unless zeroization is explicitly enforced and regression-tested.

## Objective

Close the parent task with explicit AC/conformance mapping over the merged subtask delivery:
- `#4165` (failing-first regression tests for env-secret decode buffer zeroization),
- `#4166` (zeroize implementation for decoded buffers and transient signer material).

## Scope

In scope:
- Env-secret decode buffer zeroization in strict signer precedence failure paths.
- Transient signer key-material zeroization after key construction attempts.
- Docs contract coverage and task-level lifecycle artifacts.

Out of scope:
- Hardware-backed key isolation.
- External key custody integration.

## Acceptance Criteria

- AC-1: Decode buffers and transient signer material are zeroized after use.
- AC-2: Failing-to-passing tests validate zeroization behavior.
- AC-3: Regression checks prevent secret-cleanup drift.
- AC-4: Unit/Functional/Integration/Regression coverage remains present and passing.

## Conformance Cases

- C-01 (AC-1): `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact` and `cargo test -p kamn-node signer::tests::unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material -- --exact` pass.
- C-02 (AC-2): `cargo test -p kamn-node regression_signer_secret_source_precedence_path_requires_zeroize_markers` passes.
- C-03 (AC-3): `cargo test -p kamn-node signer::tests::regression_strict_signer_secret_source_precedence_rejects_dual_private_key_envs -- --exact` passes with deterministic fail-closed reason-code behavior.
- C-04 (AC-4): `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_signer_secret_zeroization_controls -- --exact` and `cargo test -p kamn-core --test threat_control_matrix_docs matrix_contains_signer_secret_zeroization_entry_details -- --exact` pass.

## Success Metrics

- Signer env-secret precedence failures and transient signer material construction paths deterministically zeroize key buffers.
- Zeroization governance remains enforced by regression and docs-contract tests.
