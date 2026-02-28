# Spec: Issue #6244 - Restore CI Fast-Gate and E2E Live Lanes After Wave1 Extraction

- Status: Reviewed
- Priority: P1
- Parent: #6231
- Milestone: R59 Swarm Gap Closure

## Problem Statement

After merge of extraction wave1, pull-request CI lanes regressed. `ci-fast-gate` fails strict clippy checks, workspace pre-merge tests fail license-policy contract checks, and E2E live CLI smoke reports failing scenario outcomes.

## Scope

In scope:
- Remove strict clippy violations introduced by wave1 extraction changes.
- Restore workspace license policy conformance for newly introduced crates.
- Restore E2E CLI smoke live-lane contract outcomes to PASS.
- Verify fixes with commands that mirror failing CI lanes.

Out of scope:
- Broad CI policy redesign.
- Unrelated feature work outside affected lanes.

## Acceptance Criteria

### AC-1 Strict Clippy Passes
Given current workspace code,
When strict clippy runs in fast gate,
Then no `-D warnings` violations remain.

### AC-2 Workspace License Policy Contract Passes
Given workspace policy checks,
When `kamn-core` workspace license policy contract tests run,
Then the lane reports PASS with zero `license_missing` violations.

### AC-3 E2E Live CLI Smoke Reports PASS
Given external-runtime live execution in CLI scripted mode,
When the E2E harness executes configured smoke scenarios,
Then `live_execution.overall_status` is `PASS` and selected scenarios report `PASS`.

## Conformance Cases

- C-01 (AC-1, Regression): `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- C-02 (AC-2, Regression): `cargo test -p kamn-core --test workspace_license_policy_contract` passes.
- C-03 (AC-3, Integration): CI-equivalent E2E live CLI smoke invocation reports PASS.
