# Spec - Issue #4159

- Title: Story: enforce private-key zeroization and explicit signer-material requirements
- Parent: #4158
- Milestone: R27.20 Secret material zeroization and signer-rotation governance
- Status: Implemented
- Priority: P1

## Problem Statement

Signer security degrades when transient key material is not deterministically zeroized and configuration paths allow fallback behavior that masks missing signer material.

## Objective

Close the story with explicit AC/conformance mapping across completed child tasks:
- `#4161` key-material zeroization across signer lifecycle paths,
- `#4162` fallback signer-path removal and deterministic signer-material config errors.

## Scope

In scope:
- Signer env-secret/transient material zeroization checks.
- Explicit signer-material requirements with deterministic fail-closed config mapping.
- Story lifecycle artifacts and cross-task conformance evidence.

Out of scope:
- External key custody platform integration.
- Hardware-backed key lifecycle redesign.

## Acceptance Criteria

- AC-1: Private-key decode/transient buffers are deterministically zeroized in signer flows.
- AC-2: Signer configuration requires explicit material and rejects fallback paths.
- AC-3: Fail-closed reason mapping for signer config remains deterministic.
- AC-4: Unit/Functional/Integration/Regression coverage remains present and passing.

## Conformance Cases

- C-01 (AC-1): `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact` and `cargo test -p kamn-node signer::tests::unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material -- --exact` pass.
- C-02 (AC-2): `bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh` passes with fallback prohibition checks.
- C-03 (AC-3): `cargo test -p kamn-node signer::tests::regression_strict_signer_secret_source_precedence_rejects_dual_private_key_envs -- --exact` and `cargo test -p kamn-core --test service_api_ops_configuration_docs` pass.
- C-04 (AC-4): `bash scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh` passes.

## Success Metrics

- Signer secret lifecycle remains fail-closed with deterministic zeroization and explicit config requirements.
- Fallback reintroduction and signer-material mapping drift are caught by policy and docs/test contracts.
