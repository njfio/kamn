# Spec - Issue #4160

- Title: Story: validate multi-signer rotation evidence and custody policy gate integrity
- Parent: #4158
- Milestone: R27.20 Secret material zeroization and signer-rotation governance
- Status: Implemented
- Priority: P1

## Problem Statement

Multi-signer rotation readiness can be bypassed if quorum parity and custody marker evidence are not deterministically validated and fail-closed under tamper conditions.

## Objective

Close the story with AC/conformance mapping over completed child task `#4163`, which delivered rotation preflight marker/quorum checks and deterministic custody reason mapping.

## Scope

In scope:
- Rotation preflight marker completeness and quorum parity validation.
- Deterministic custody reason taxonomy mapping and docs marker governance.
- Story lifecycle artifacts and conformance evidence.

Out of scope:
- External approval workflow automation.
- Enterprise custody platform integration.

## Acceptance Criteria

- AC-1: Rotation preflight checks enforce required markers and quorum policy.
- AC-2: Drift/mismatch emits deterministic fail-closed reason markers.
- AC-3: Failing-to-passing tests validate marker/quorum/custody behavior.
- AC-4: Unit/Functional/Integration/Regression coverage remains present and passing.

## Conformance Cases

- C-01 (AC-1): `bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh` passes.
- C-02 (AC-2): same checker test passes with deterministic custody reason mappings.
- C-03 (AC-3): `bash scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh` passes.
- C-04 (AC-4): `cargo test -p kamn-core --test kolme_devnet_ops_docs` and `cargo test -p kamn-core --test release_gonogo_checklist_docs` pass.

## Success Metrics

- Rotation readiness evidence remains fail-closed with deterministic marker/quorum/custody reason outputs.
- Docs contracts enforce rotation preflight marker governance in CI.
