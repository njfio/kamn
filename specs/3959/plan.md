# Issue #3959 Plan

- Issue: #3959
- Status: Completed
- Spec: `specs/3959/spec.md`

## Implementation Approach
1. Add a deterministic reason-taxonomy marker for runtime signer key-source policy.
2. Add a RED regression test asserting fallback signer secret env is rejected at the runtime key-source policy gate.
3. Update `enforce_kolme_live_signer_key_source_policy` to detect fallback env presence and fail closed with deterministic reason code.
4. Re-run targeted unit/functional/integration/regression tests.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/arg_and_signer_policy_tests.rs`

## Risks and Mitigations
- Risk: broadening policy gate semantics could disrupt existing strict env-local tests.
  - Mitigation: preserve existing env-local reason code path and add focused regression for fallback path.
- Risk: env-var state leakage across tests.
  - Mitigation: gate new test with `signer_env_lock()` and `EnvVarGuard` around fallback env marker.

## Contracts and Interfaces
- Deterministic reason code: `fallback_signer_secret_present_violation`.
- Deterministic taxonomy marker: `runtime_signer_key_source_policy_reason_codes_csv`.
- No wire/API/schema changes.

## Verification Strategy
- RED: new fallback policy regression test fails before implementation.
- GREEN: runtime policy gate update makes RED test pass.
- REGRESSION: existing strict env-local and managed-external policy tests remain green.
