# Spec — Issue #4166

- Title: Subtask: implement zeroize handling for decoded key buffers and transient signer material
- Parent: Parent task: #4161
- Milestone: R27.20 Secret material zeroization and signer-rotation governance
- Status: Reviewed
- Priority: P1

## Objective

Implement deterministic zeroization for signer transient material across env-secret precedence checks and signer key decode/construction paths.

## Problem Statement

Signer secret handling currently relies on partial zeroization and does not explicitly scrub every transient buffer on all rejection paths.

## Scope

In scope:
- explicit zeroization for env-secret buffers when strict precedence checks fail
- explicit zeroization for transient key-material buffers used during signer key construction
- docs updates mapping zeroization controls to operations and threat governance references

Out of scope:
- external signer custody system integration
- runtime protocol/schema changes

## Acceptance Criteria

- AC-1: Env-secret buffers are explicitly zeroized before returning strict precedence violations.
- AC-2: Transient signer key-material buffers are explicitly zeroized after key construction attempts.
- AC-3: Ops and threat-model docs expose deterministic zeroization control markers and regression references.

## Conformance Cases

- C-01 (AC-1, Unit): `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact`
- C-02 (AC-2, Unit): `cargo test -p kamn-node signer::tests::unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material -- --exact`
- C-03 (AC-3, Conformance): `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_signer_secret_zeroization_controls -- --exact`
- C-04 (AC-3, Conformance): `cargo test -p kamn-core --test threat_control_matrix_docs matrix_contains_signer_secret_zeroization_entry_details -- --exact`

## Success Metrics / Signals

- Signer secret handling rejects fail closed with deterministic reason codes and no secret echo.
- Zeroization controls remain enforced by unit and docs-contract regression coverage.

