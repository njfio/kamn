# Issue #5322 Spec

- Title: Stabilize signer env guard tests by unifying test env lock across `main_tests` and `signer` module tests
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
`kamn-node` signer-env-mutating tests are flaky under parallel execution because they do not all share one lock for process-wide `KAMN_KOLME_LIVE_*` env variables. Separate lock domains allow races and lock poisoning side effects.

## Acceptance Criteria
- AC-1: Signer-env-mutating tests in `main_tests` and `signer.rs` test module use one shared crate-wide lock source.
- AC-2: Poisoned lock acquisition paths fail open to guard recovery for signer env lock acquisition used by runtime/main tests.
- AC-3: The flaky regression `regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch` remains deterministic under parallel execution.
- AC-4: Existing signer reason-code assertions remain unchanged.

## Scope
In scope:
- Test-only lock unification for signer env mutation paths.
- Test helper wiring updates in `main.rs`, `main_tests.rs`, and `signer.rs` tests.
- Targeted `kamn-node` test verification.

Out of scope:
- Introducing third-party serialization crates.
- Changing production signer behavior.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | parallel `kamn-node` test run touching signer env tests | no cross-module env race caused by distinct locks |
| C-02 | AC-2 | Regression | lock acquisition after poisoned signer lock scenario | acquisition succeeds with recovery path (no panic cascade) |
| C-03 | AC-3 | Regression | run flaky signer regression in parallel stress context | deterministic pass |
| C-04 | AC-4 | Functional | signer reason-code assertions | unchanged expected reason markers |

## Test Mapping
- `cargo test -p kamn-node main_tests::signer_tests::regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch -- --exact --test-threads=16`
- `cargo test -p kamn-node main_tests::runtime_tests::unit_kolme_live_local_signer_override_marker_parses_boolean_values -- --exact --test-threads=16`
- `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact --test-threads=16`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::regression_service_api_env_lock_recovers_from_signer_lock_poison -- --exact`

## Success Metrics
- No nondeterministic failures in targeted signer-env tests under parallel execution.
- Lock poisoning no longer cascades into unrelated signer-env tests.
