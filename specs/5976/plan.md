# Plan: Issue #5976

## Approach
- Reuse existing service API relay/persistence integration suite and harden matrix completeness checks.
- Extend e2e-live workflow contract tests for required jobs/env/evidence assertions.
- Add a small deterministic mapping artifact (or test fixture) connecting R57 high gaps to current guard checks.

## Affected Modules (Expected)
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-e2e-harness/tests/*workflow*`
- `scripts/ci/*` and/or docs fixture used for evidence mapping

## Risks / Mitigations
- Risk: evidence mapping drifts from actual tests.
  Mitigation: validate mapping entries against executable identifiers in test suite.

## Interfaces / Contracts
- Live-E2E workflow contract markers.
- High-gap mapping artifact schema.
