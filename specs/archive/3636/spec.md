# Issue #3636 Spec

- Title: `Task: extract signer adapter module for crypto and key-source operations`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Signer cryptographic and key-source paths were interleaved with policy and orchestration logic, increasing blast radius for changes.

## Scope
In:
- Move adapter-level signing and key-source behavior into dedicated signer adapter ownership.
- Preserve runtime behavior and outputs through parity checks.
- Lock boundary ownership with deterministic drift checks.

Out:
- Policy/quorum decision logic redesign.

## Acceptance Criteria
- AC-1: signer adapter module owns crypto + key-source paths.
- AC-2: public API boundaries are enforced by drift contracts.
- AC-3: signing parity remains stable across integration lanes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1/AC-2 | Functional | `cargo test -p kamn-node --test signer_adapter_boundary_contract` | adapter boundary contract passes |
| C-02 | AC-2 | Functional | `cargo test -p kamn-node --test signer_extraction_budget_contract` | extraction ownership budget guards pass |
| C-03 | AC-3 | Integration | `bash scripts/kolme/test_run_signature_parity_contract_lane.sh` | signature parity lane passes |
| C-04 | AC-3 | Regression | `bash scripts/kolme/test_check_signature_parity_policy.sh` | parity policy marker checks pass |

## Test Mapping
- `crates/kamn-node/tests/signer_adapter_boundary_contract.rs`
- `crates/kamn-node/tests/signer_extraction_budget_contract.rs`
- `scripts/kolme/test_run_signature_parity_contract_lane.sh`
- `scripts/kolme/test_check_signature_parity_policy.sh`

## Success Metrics
- Adapter ownership is explicit and enforced.
- Signing parity lanes remain deterministic.
