# Issue #4097 Plan

## Summary
Add overload governance docs/runbook parity and go/no-go marker contracts by extending existing docs-contract harnesses in `kamn-core` tests. Keep shell surface unchanged.

## Affected Areas
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `specs/4097/{spec.md,plan.md,tasks.md}`

## Approach
1. Add overload docs parity marker block in `docs/ci/strategy.md` with deterministic taxonomy/version/csv/remediation markers and guard commands.
2. Add overload remediation marker references in `docs/ops/configuration.md` under overload stress matrix controls.
3. Extend `ci_strategy_docs.rs` with overload parity tests:
   - marker presence contract,
   - source parity against ops doc + runner script marker values,
   - remediation coverage for every declared reason code.
4. Extend `service_api_ops_configuration_docs.rs` with overload remediation marker contract checks.
5. Run targeted red/green test loop and finalize issue/PR evidence.

## Risks and Mitigations
- Risk: marker taxonomy mismatch between strategy and runner.
  - Mitigation: parity test directly checks runner source marker strings.
- Risk: remediation entries drift from reason code list.
  - Mitigation: iterate parsed reason code CSV and assert remediation map key presence.
- Risk: accidental shell LOC growth.
  - Mitigation: no shell/workflow changes in scope.

## Interfaces / Contracts
- Docs marker contract keys:
  - `overload_docs_parity_reason_taxonomy_version`
  - `overload_docs_parity_reason_codes_csv`
  - `overload_docs_parity_runner_schema_version`
  - `overload_docs_parity_remediation.<reason_code>`
- Runner source contract: `scripts/ci/run_daemon_os_signal_stress_matrix.sh` reason-code assignments and report schema marker.

## ADR
Not required (no architecture, dependency, or protocol shape changes).
