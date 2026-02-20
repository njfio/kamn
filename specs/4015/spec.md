# Issue #4015 Spec

- Title: Subtask: define journal-wal schema fixtures and commit-boundary marker contracts
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Problem Statement

File-backed snapshot stores for channel, message lifecycle, and task operations already append deterministic journal records and emit deterministic recovery reason codes, but there is no single fixture contract that locks the journal schema and commit-boundary marker map. Without this fixture + docs parity contract, marker drift can silently break replay/recovery governance.

## Scope

In scope:
- Add a deterministic runtime fixture matrix for journal/WAL entry schema and commit-boundary marker map.
- Add contract tests that validate fixture schema, journal record structure, and recovery marker determinism against exported store behavior.
- Update runtime network docs with fixture path, schema markers, and store-to-marker mapping.
- Extend docs contract tests to fail closed on marker drift.

Out of scope:
- New persistence backends.
- Storage engine migration.
- Runtime write-path behavior changes beyond contract coverage and documentation.

## Acceptance Criteria

- AC-1: Fixture `fixtures/runtime/journal_wal_commit_boundary_fixture_matrix.txt` defines deterministic journal entry schema markers and per-store commit-boundary marker mappings.
- AC-2: Contract tests verify journal records emitted by file-backed snapshot stores conform to `entry|1|<payload_hex>` structure.
- AC-3: Contract tests verify deterministic commit-boundary marker helper behavior for empty, clean, repaired-corrupt-payload, and corrupt-tail paths across channel/message/task stores.
- AC-4: `docs/foundation/runtime-network.md` includes a journal/WAL commit-boundary marker map section, and docs tests fail closed on drift.
- AC-5: Required unit, functional, integration, and regression tests are present and passing.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | fixture metadata lines | schema/taxonomy/columns markers parse deterministically |
| C-02 | AC-2 | Functional | valid snapshots written via file stores | journal records match `entry|1|<payload_hex>` with deterministic prefix/version/hex encoding |
| C-03 | AC-3 | Integration | empty store recovery | `recover_latest_and_repair().reason_code()` equals fixture empty marker |
| C-04 | AC-3 | Integration | clean store recovery after valid write | `recover_latest_and_repair().reason_code()` equals fixture clean marker |
| C-05 | AC-3 | Integration | invalid payload file with no journal | recovery returns repaired marker from fixture |
| C-06 | AC-3/AC-4 | Regression | corrupted journal tail + docs marker checks | corrupt-tail prefix and docs marker map remain deterministic (`Regression: #4015`) |

## Test Mapping

- `cargo test -p kamn-core --test journal_wal_commit_schema_contract -- --nocapture`
- `cargo test -p kamn-core --test runtime_network_docs doc_contains_journal_wal_commit_boundary_marker_rules -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`

## Success Metrics / Observable Signals

- Fixture-backed journal schema and commit-boundary marker map is deterministic and test-enforced.
- File-backed channel/message/task recovery markers are parity-checked against fixture contracts.
- Runtime-network docs and docs tests remain synchronized with fixture markers.
