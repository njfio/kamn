# Issue #5199 Plan

- Issue: #5199
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Approach
1. Inventory all `signer_env_lock()` lock callsites in `main_tests` and classify acquisition behavior.
2. Replace poison-propagating `.expect(...)` lock acquisition with `lock_signer_env_guard()` in CLI and service-api test modules.
3. Add a regression test that intentionally poisons the shared signer env lock and then acquires it via `lock_signer_env_guard()` to assert recovery.
4. Run targeted `kamn-node` test subsets for CLI and signer suites to validate no cascade.

## Affected Modules / Files
- `crates/kamn-node/src/main_tests/cli_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/5199/spec.md`
- `specs/5199/tasks.md`

## Risks and Mitigations
- Risk: Intentional lock poisoning test introduces order-dependent side effects.
  - Mitigation: Ensure all signer env lock acquisition callsites use poison-recovery helper before adding regression test.
- Risk: Hidden direct lock acquisition remains and fails after poison regression executes.
  - Mitigation: source-scan `main_tests` for any remaining `signer_env_lock().lock().expect(...)` patterns and fix them in this change.

## Interfaces / Contracts
- Existing lock contract:
  - `lock_signer_env_guard()` must be the sole signer env lock acquisition path for env-mutating tests.
- Regression contract marker:
  - `regression_signer_env_lock_recovers_after_poison` validates post-poison guard acquisition behavior.
