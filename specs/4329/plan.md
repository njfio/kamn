# Plan — #4329

Status: Reviewed

## Approach

- Extend `scripts/runtime/local_full_stack_integration_live_contract.py` with runtime module-boundary parity checker output and policy validation.
- Add RED tests in existing runtime contract-lane shell tests for new markers/reason mapping.
- Keep implementation additive to preserve existing runtime phase parity contracts.
- Update runtime architecture + CI strategy docs and docs contract tests.

## Affected Areas

- `scripts/runtime/local_full_stack_integration_live_contract.py`
- `scripts/runtime/test_validate_local_full_stack_integration_live.sh`
- `scripts/runtime/test_check_local_full_stack_integration_live_policy.sh`
- `scripts/runtime/test_validate_local_full_stack_integration_live_contract_lane.sh`
- `docs/architecture/runtime.md`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/runtime_architecture_docs.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations

- Risk: marker drift breaks existing policy contract lanes.
  - Mitigation: additive fields and deterministic constants; preserve existing phase parity markers.
- Risk: checker creates high-cost CI behavior.
  - Mitigation: static source-boundary checks only in ci-smoke paths; no heavy nested command execution.

## Interfaces and Contracts

- New runtime module-boundary taxonomy markers and normalized reason outputs.
- Policy output includes deterministic module-boundary reason taxonomy version/codes/value.
