# Issue #5301 Plan

## Objective
Implement deterministic convergence evidence projection markers and wire them into daemon bootstrap report output for promotion gating under story `#5254`.

## Approach
1. Add a small convergence projection contract/helper in `kamn-node` runtime orchestration that:
   - Accepts per-class convergence booleans for schema, error-path, concurrency, performance, and cost gates.
   - Emits deterministic taxonomy/version and reason code markers.
   - Fails closed (`no_go`) if any class fails.
2. Invoke the helper from daemon execution completion and attach projection markers to `DaemonExecution`.
3. Project markers through `report_builder.rs` and `report_render.rs` (text + json output).
4. Add tests for pass/fail decision matrix and regression stability.
5. Add docs marker contract coverage in `docs/ops/configuration.md` and docs-contract tests.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/report_builder.rs`
- `crates/kamn-node/src/report_render.rs`
- `crates/kamn-node/src/main_tests/daemon_tests.rs`
- `crates/kamn-node/src/main_tests/report_tests.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ops/configuration.md`

## Risks and Mitigations
- Risk: marker drift between daemon struct, builder, and renderer.
  - Mitigation: integration test asserts marker presence in report json/text.
- Risk: ambiguous fail reason ordering.
  - Mitigation: deterministic reason-code ordering and regression test for stable output.
- Risk: shell-surface regression from governance changes.
  - Mitigation: keep implementation Rust-only; no scripts/workflows/templates modified.

## Contracts and Interfaces
- New convergence marker fields in `DaemonExecution` and `NodeBootstrapReport`.
- Stable constants for:
  - convergence reason taxonomy version
  - supported reason code csv
  - decision enum string markers (`go`, `no_go`)

## ADR
No ADR required: no new dependency, protocol, or wire-format change.
