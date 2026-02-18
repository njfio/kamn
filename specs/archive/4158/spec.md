# Spec - Issue #4158

- Title: Epic: R27.20 close secret material zeroization and signer-rotation governance gaps
- Parent: #3812
- Milestone: R27.20 Secret material zeroization and signer-rotation governance
- Status: Implemented
- Priority: P1

## Problem Statement

Signer security and rotation readiness require coordinated enforcement across secret zeroization, explicit signer-material requirements, and deterministic preflight custody/quorum evidence governance.

## Objective

Close the epic with explicit AC/conformance mapping across completed child stories:
- `#4159` signer zeroization and explicit signer-material/fallback-prohibition governance,
- `#4160` multi-signer rotation preflight marker/quorum/custody governance.

## Scope

In scope:
- Secret material zeroization contract coverage in signer lifecycle paths.
- Deterministic explicit signer-material configuration/fallback prohibition checks.
- Rotation preflight quorum parity and custody marker governance.
- Epic-level lifecycle artifacts and consolidated verification.

Out of scope:
- External key custody platform integration.
- New feature/protocol work unrelated to signer governance.

## Acceptance Criteria

- AC-1: Secret material zeroization remains enforced in signer precedence and transient key paths.
- AC-2: Signer configuration remains explicit-only with deterministic fail-closed mapping.
- AC-3: Rotation preflight marker/quorum/custody governance remains deterministic and fail-closed.
- AC-4: Unit/Functional/Integration/Regression coverage remains present and passing across epic governance surfaces.

## Conformance Cases

- C-01 (AC-1): `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact` and `cargo test -p kamn-node signer::tests::unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material -- --exact` pass.
- C-02 (AC-2): `bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh` and `cargo test -p kamn-core --test service_api_ops_configuration_docs` pass.
- C-03 (AC-3): `bash scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh`, `cargo test -p kamn-core --test kolme_devnet_ops_docs`, and `cargo test -p kamn-core --test release_gonogo_checklist_docs` pass.
- C-04 (AC-4): combined checks above remain green and deterministic.

## Success Metrics

- Signer secret lifecycle and rotation readiness governance stay fail-closed with deterministic reason-marker outputs.
- Epic closure is fully auditable in-repo with AC->test traceability.
