# Issue #4321 Spec

- Title: `Subtask: add red tests for persisted block digest-finality mismatch rejection and tamper detection`
- Status: `Reviewed`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-and-durable-block-pipeline-governance/index.md`
- Parent: `#4314`

## Problem Statement
Durable block commit replay validation must fail closed when persisted artifacts drift (digest/checkpoint/finality lineage) or are tampered.

## Scope
In:
- Add deterministic persisted replay tamper tests for digest/checkpoint mismatch outcomes.
- Add deterministic persisted artifact tamper detection tests for commit replay evidence.
- Update release go/no-go checklist references for block mismatch/tamper failure modes.

Out:
- Consensus/fork-choice algorithm redesign.
- Storage schema redesign.

## Acceptance Criteria
- AC-1: tests fail when persisted payload-digest mismatch is accepted.
- AC-2: tests fail when persisted checkpoint/finality lineage mismatch is accepted.
- AC-3: regression matrix preserves deterministic rejection reason codes for tamper cases.
- AC-4: release go/no-go checklist documents block mismatch/tamper failure modes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `cargo test -p kamn-core --test block_commit_persistence_tamper_matrix unit_replay_rejects_persisted_payload_digest_mismatch_reason_code -- --exact` | replay drift fails closed with `canonical_replay_payload_digest_mismatch` |
| C-02 | AC-2 | Functional | `cargo test -p kamn-core --test block_commit_persistence_tamper_matrix functional_replay_rejects_persisted_checkpoint_missing_reason_code -- --exact` | checkpoint/finality drift fails closed with `canonical_replay_checkpoint_missing` |
| C-03 | AC-3 | Integration | `cargo test -p kamn-core --test block_commit_persistence_tamper_matrix integration_replay_tamper_matrix_emits_deterministic_reason_codes -- --exact` | mismatch/tamper matrix emits stable reason codes |
| C-04 | AC-3 | Regression | `cargo test -p kamn-core --test block_commit_persistence_tamper_matrix regression_replay_height_mismatch_reason_code_stable -- --exact` | height drift stays mapped to deterministic reason code |
| C-05 | AC-3 | Performance | `cargo test -p kamn-core --test block_commit_persistence_tamper_matrix performance_replay_tamper_matrix_stays_within_local_budget -- --exact` | bounded runtime for replay tamper matrix |
| C-06 | AC-4 | Docs | `cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_block_commit_persistence_mismatch_tamper_gate -- --exact` | release checklist includes block mismatch/tamper failure markers |

## Test Mapping
- `crates/kamn-core/tests/block_commit_persistence_tamper_matrix.rs` (new)
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `docs/foundation/release-gonogo-checklist.md`

## Success Metrics
- Persisted replay mismatch/tamper reason codes are guarded by deterministic tests.
- Release checklist includes stable block mismatch/tamper failure markers for go/no-go policy lanes.
