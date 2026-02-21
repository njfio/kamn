# Issue #5418 Plan — Daemon Tests Phase-2 Decomposition

## Approach
1. Split `daemon_tests.rs` by moving the large live-postgres topology contract slice into `daemon_tests/live_postgres_topology_contract_tests.rs` and include it from the root test module.
2. Add explicit phase-2 decomposition marker comment + include routing in `daemon_tests.rs`.
3. Extend extraction contract test suite (`main_module_extraction_contract.rs`) to assert daemon-tests shell budget + include routing.
4. Update ops docs decomposition markers and docs-contract assertions for phase-2 module path/target.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests.rs` (new)
- `crates/kamn-node/tests/main_module_extraction_contract.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks / Mitigations
- Risk: test path name drift after extraction.
  - Mitigation: use `include!` into the same module namespace to preserve `main_tests::daemon_tests::*` paths.
- Risk: docs marker drift.
  - Mitigation: update docs and fail-closed docs-contract assertions in the same change.

## Interfaces / Contracts
- Keep test invocation paths stable (`main_tests::daemon_tests::...`).
- Add decomposition marker: `daemon_tests structural budget shell phase2; ...`.
- Add docs phase-2 marker key/value contract for module path + max-line target.

## Validation Strategy
- Red: add/adjust extraction + docs-contract assertions for phase-2 markers.
- Green: perform include-based extraction and docs updates until targeted tests pass.
- Verify: run targeted suites, formatting, and strict lint.
