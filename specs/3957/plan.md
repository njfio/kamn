# Issue #3957 Plan

- Issue: #3957
- Status: Implemented

## Approach
1. Add table-driven quorum matrix fixture coverage in existing signer preflight tests (`crates/kamn-node/src/signer.rs`).
2. Add integration-level matrix harness in `crates/kamn-node/src/main_tests/signer_tests.rs` to exercise runtime-facing preflight entrypoint with deterministic markers.
3. Document matrix fixture contract markers in `docs/ops/configuration.md` and enforce via `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.

## Affected Modules
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks and Mitigations
- Risk: Matrix fixtures can become brittle if env setup leaks between cases.
  Mitigation: use existing signer env lock and case-scoped `EnvVarGuard`s for deterministic isolation.
- Risk: Reason marker assertions may diverge from policy taxonomy.
  Mitigation: assert exact marker substrings already listed in signer policy taxonomy and docs contracts.
- Risk: Test runtime expansion in CI fast gate.
  Mitigation: keep matrix compact and deterministic; no network/dependency additions.

## Interfaces / Contracts
- Existing API retained: `evaluate_kolme_live_signer_preflight_readiness(...)` and `enforce_kolme_live_signer_preflight(...)`.
- No wire-format, schema, or dependency changes.
