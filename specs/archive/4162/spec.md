# Spec - Issue #4162

- Title: Task: remove fallback signer-key paths and enforce explicit deterministic configuration contracts
- Parent: #4159
- Milestone: R27.20 Secret material zeroization and signer-rotation governance
- Status: Implemented
- Priority: P1

## Problem Statement

Fallback signer-key behavior can mask missing signer material and weaken custody controls unless signer preflight policy checks fail closed with deterministic configuration error markers.

## Objective

Close the parent task by mapping ACs to merged subtask delivery:
- `#4167` failing-first fallback prohibition/config requirement tests,
- `#4168` deterministic signer-material config error mapping and docs contracts.

## Scope

In scope:
- Fallback signer-key path prohibition in deployment preflight policy checks.
- Deterministic missing/invalid signer-material configuration reason markers.
- Parent task lifecycle artifacts and conformance mapping.

Out of scope:
- Key distribution workflow redesign.
- Unrelated runtime feature additions.

## Acceptance Criteria

- AC-1: Fallback signer-key paths are removed from active policy paths.
- AC-2: Missing signer material yields deterministic configuration errors.
- AC-3: Regression tests prevent fallback-path reintroduction.
- AC-4: Unit/Functional/Integration/Regression coverage remains present and passing.

## Conformance Cases

- C-01 (AC-1): `bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh` passes and enforces fallback prohibition reason mappings.
- C-02 (AC-2): same checker test passes with deterministic `signer_secret_missing` and `signer_secret_invalid_hex` mappings.
- C-03 (AC-3): `bash scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh` passes and keeps fallback/prohibition checks fail-closed.
- C-04 (AC-4): `cargo test -p kamn-core --test service_api_ops_configuration_docs` passes with signer-material requirement marker contracts.

## Success Metrics

- Preflight signer configuration remains explicit-only, with deterministic reason taxonomy output for missing/invalid signer material.
- Fallback signer-key regressions are caught by policy and docs-contract tests.
