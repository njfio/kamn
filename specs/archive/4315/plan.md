# Issue #4315 Plan

- Issue: `#4315`
- Status: `Completed`

## Approach
- Add explicit regression coverage for repeated concurrency-limit pressure rounds to prove deterministic fail-closed reason outputs.
- Add functional projection coverage for the backpressure reason-code set used by async limiter/admission paths.
- Add a docs contract test for `docs/ops/configuration.md` and update the doc with async API backpressure failure-mode markers.

## Affected Modules
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks and Mitigations
- Risk: concurrency fixture can be flaky under scheduler variance.
- Mitigation: use bounded worker count + barrier synchronization + deterministic assertions focused on fail-closed reason fields.
- Risk: docs drift after merge.
- Mitigation: add dedicated docs test with explicit marker assertions.

## Interface Contract
- Additive test/docs changes only; no API wire-format changes.

## ADR
- Not required.
