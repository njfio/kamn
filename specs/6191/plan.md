# Plan: Issue 6191 - Extract Shared Snapshot/Journal Helpers

- Issue: #6191
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Introduce `snapshot_journal` module with shared helpers:
   - default journal path derivation
   - append journal record
   - parse journal record
   - decode journal hex payload
2. Rewire `message_lifecycle`, `channel_models`, and `task_operations` to use shared helpers.
3. Keep module-local error taxonomy and corrupt-tail reason strings unchanged.
4. Validate parity with existing corrupt-tail regression tests.

## Affected Modules

- `crates/kamn-core/src/snapshot_journal.rs`
- `crates/kamn-core/src/message_lifecycle.rs`
- `crates/kamn-core/src/channel_models.rs`
- `crates/kamn-core/src/task_operations.rs`
- `crates/kamn-core/src/lib.rs`

## Risks and Mitigations

1. Risk: helper extraction changes replay parsing behavior.
   - Mitigation: reuse existing journal tests that assert corrupt-tail reason contracts.
2. Risk: broad kamn-core test matrix is heavy in constrained environments.
   - Mitigation: run focused `--lib` regression lanes covering touched behavior.
