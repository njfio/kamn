# Issue #4016 Spec

- Title: Subtask: add partial-write fault injection regressions and atomic-commit boundary assertions
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Problem Statement

File-backed channel/message/task snapshot stores maintain journal records for deterministic recovery, but partial-write faults need explicit cross-store regression coverage to prove commit-boundary behavior. In particular, partial snapshot-file writes must not silently override valid journal commits, and partial journal-tail writes must fail closed with stable markers.

## Scope

In scope:
- Add deterministic partial-write fault fixture matrix for channel/message/task snapshot stores.
- Add contract tests for partial snapshot-file write recovery, partial journal-tail fail-closed behavior, and no-journal corrupt snapshot repair behavior.
- Strengthen file-store read path semantics so valid journal commits are authoritative under partial snapshot-file corruption.
- Update `docs/ops/configuration.md` with partial-write drill inputs and expected outcomes.
- Add docs-contract assertions for ops marker parity.

Out of scope:
- External DB transaction engine migration.
- New chaos framework.
- Non-snapshot persistence subsystems.

## Acceptance Criteria

- AC-1: Partial-write fault-injection regressions cover deterministic partial snapshot-file and journal-tail scenarios for channel/message/task snapshot stores.
- AC-2: Atomic-commit boundary assertions guarantee valid journal commits remain authoritative when snapshot file payloads are partially written/corrupted.
- AC-3: Corrupt partial journal tail fails closed with deterministic store-specific reason markers.
- AC-4: `docs/ops/configuration.md` documents partial-write drill fixture markers, fault inputs, and expected outcomes; docs tests fail closed on drift.
- AC-5: Unit, Functional, Integration, Regression, and Performance tests for this contract are present and passing.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1/AC-2 | Functional | valid journal commit + partially written snapshot payload | `read_latest` / recovery returns latest journal commit (no silent partial state) |
| C-02 | AC-1/AC-3 | Integration | valid journal + corrupted tail entry | fail closed with `<store>_snapshot_journal_corrupt_tail:<line-index>` |
| C-03 | AC-1 | Integration | partial/corrupt snapshot payload without journal | deterministic repaired outcome marker |
| C-04 | AC-1 | Unit | partial-write fixture parser | schema/taxonomy/columns markers parse deterministically |
| C-05 | AC-4 | Regression | ops docs marker assertions | docs parity drift fails closed (`Regression: #4016`) |
| C-06 | AC-5 | Performance | full fault matrix execution | bounded runtime under deterministic CI budget |

## Test Mapping

- `cargo test -p kamn-core --test journal_wal_partial_write_fault_contract -- --nocapture`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_journal_wal_partial_write_fault_injection_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`

## Success Metrics / Observable Signals

- Cross-store partial-write contracts are deterministic and fixture-driven.
- Valid journal commits remain authoritative under partial snapshot-file corruption.
- Ops runbook markers remain synchronized with fixture + expected outcome taxonomy.
