# Plan: Issue #5979

## Approach
- Expand existing service API endpoint relay/delivery integration tests where needed.
- Expand e2e-live workflow contract tests for required marker coverage.
- Add and validate a compact evidence matrix fixture.

## Affected Modules
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-e2e-harness/tests/phase4i_ci_workflow_contract.rs`
- `crates/kamn-e2e-harness/tests/docs_contract_release_group.rs` or dedicated mapping contract file

## Risks / Mitigations
- Mapping artifact staleness.
  Mitigation: contract test enumerates required gaps and verifies referenced checks exist.

## Interfaces / Contracts
- Evidence matrix schema and validation rules.
