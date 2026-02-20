# Issue #4013 Plan

## Implementation Approach

1. Write RED tests first:
- add a new integration test module `crates/kamn-core/tests/cross_store_replay_consistency.rs`.
- include failing assertions for deterministic status/reason/class/taxonomy behavior.
- run targeted test command to capture RED evidence.

2. Implement checker module:
- add `crates/kamn-core/src/cross_store_replay_consistency.rs`.
- define deterministic report/status/class contracts.
- implement evaluation logic for presence/schema/runtime-continuity/cardinality divergence.
- expose stable taxonomy markers (`reason_taxonomy_version`, `reason_codes_csv`).

3. Wire public exports:
- add module and re-exports in `crates/kamn-core/src/lib.rs`.

4. Add docs contracts:
- extend `docs/foundation/runtime-network.md` with checker marker section.
- add/extend docs-contract test in `crates/kamn-core/tests/runtime_network_docs.rs`.

5. Green/verification:
- rerun targeted checker tests and docs-contract tests.
- run `cargo fmt --check` and scoped `cargo clippy`.

## Affected Modules

- `crates/kamn-core/src/cross_store_replay_consistency.rs` (new)
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/cross_store_replay_consistency.rs` (new)
- `crates/kamn-core/tests/runtime_network_docs.rs`
- `docs/foundation/runtime-network.md`

## Risks and Mitigations

- Risk: taxonomy churn creates flaky policy wiring.
  - Mitigation: centralize taxonomy version and reason-code CSV constants; add regression tests for exact marker stability.
- Risk: overfitting checker to one store shape.
  - Mitigation: evaluate normalized snapshots only (runtime/channel/message/task) and keep deterministic checks data-shape based.
- Risk: docs drift from implementation.
  - Mitigation: fail-closed docs contract assertion for checker markers.

## Interfaces / Contracts

- New API (planned):
  - `evaluate_cross_store_replay_consistency(...) -> CrossStoreReplayConsistencyReport`
  - `cross_store_replay_reason_taxonomy_version() -> &'static str`
  - `cross_store_replay_reason_codes_csv() -> &'static str`
- Report contract includes deterministic status, reason code, divergence class, and taxonomy marker.

## ADR

- Not required: no dependency/protocol/wire-format change.
