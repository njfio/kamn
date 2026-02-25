# Spec: Issue #5931 - Task: Harden managed signer execution and secret env handling

- Issue: #5931
- Status: Implemented
- Type: task
- Priority: P1
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5918

## Problem Statement
Managed signer path uses sh -c and inherits full environment, including sensitive values.

## Scope
In scope:
- Switch to argv-safe process execution and explicit env allowlist for child process.

Out of scope:
- New managed signer backend capability beyond hardening scope.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: No managed-signer execution path uses shell command interpolation.
- AC-2: Child process environment excludes signer private key envs by default.
- AC-3: Security tests prove command-injection payloads and env leakage attempts fail.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Regression, AC-1/AC-3): `signer::managed_backend::tests::regression_managed_external_backend_command_injection_payload_is_not_interpreted` proves shell-injection payloads are not interpreted by managed signer backend execution.
- C-02 (Regression, AC-2/AC-3): `signer::managed_backend::tests::regression_managed_external_backend_scrubs_signer_secret_env_for_child_process` proves signer secret env markers are not inherited by child process execution.
- C-03 (Integration, AC-1/AC-4): `main_tests::signer_tests::integration_kolme_live_managed_external_adapter_provenance_consumed_by_signer_selection` verifies managed-external signing path remains functional through hardened command execution.
- C-04 (Conformance, AC-4): `cargo test -p kamn-node --bin kamn-node signer::managed_backend::tests` and `cargo test -p kamn-node --bin kamn-node main_tests::signer_tests:: -- --nocapture`.
- C-05 (Verify, AC-4): `cargo clippy -p kamn-node --bin kamn-node -- -D warnings` and `cargo fmt --check`.

## Success Metrics / Observable Signals
- Managed signer subprocess execution uses argv-tokenized direct spawn path (no shell interpolation).
- Managed signer child-process env is scrubbed to allowlist + signer context markers.
- Security regression tests for injection payload and signer-secret env leakage are green.
- Signer integration/doc-contract suites remain green under hardened execution path.


## Required Test Categories
- Unit: command builder and env scrubber
- Functional: managed signer invocation success/failure
- Integration: signer backend with controlled child process
- Regression: sh -c path removed
- Performance: signing invocation overhead bounded

## Dependencies
- #5918
