# Issue #3882 Spec

- Title: Subtask: implement native cutover-rollback evidence bundle lane
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Activation readiness cannot be audited without deterministic bundle artifacts for cutover and rollback paths.

## Scope
In:
- Add cutover and rollback evidence bundle lane.

Out:
- Policy checker logic.

## Acceptance Criteria
- AC-1:  Evidence bundle lane emits stable schema markers.
- AC-2:  Bundle includes cutover and rollback outcome checkpoints.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional | `bash scripts/cutover/test_generate_cutover_rollback_evidence_bundle.sh` | bundle payload enforces deterministic `schema_version=kamn.cutover.rollback-evidence.v1` and stable summary markers (`final_decision`, `rollback_hash_match`, `evidence_complete`). |
| C-02 | AC-2 | Functional/Integration | `bash scripts/cutover/test_generate_cutover_rollback_evidence_bundle.sh` | rollback checkpoint markers remain deterministic (`rollback.trigger_status`, `rollback.checkpoint_state`, `rollback.failed_checkpoint_id`) and NO-GO checkpoint/reason paths are validated. |
| C-03 | AC-3 | Regression | `cargo test -p kamn-core --test kolme_devnet_ops_docs` | next-steps docs regression suite remains green with cutover rollback marker-surface additions. |

## Test Mapping
- `scripts/cutover/test_generate_cutover_rollback_evidence_bundle.sh`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
