# Validator Lifecycle and Quorum Reconfiguration (Issues #194 / #195 / #523)

This document captures the first implementation slice for validator onboarding, offboarding, and quorum updates.

## Scope Delivered
- Added `crates/kamn-core/src/validator_lifecycle.rs` with:
  - `ValidatorLifecycleManager` for:
    - `onboard_validator(...)`
    - `offboard_validator(...)`
    - `reconfigure_quorum(...)`
    - `rollback_last_transition(...)`
  - transition proof model:
    - `ValidatorTransitionProof`
  - transition audit surfaces:
    - `ValidatorTransitionRecord`
    - `ValidatorTransitionKind`
    - `transition_history()`
  - validator set snapshot model:
    - `ValidatorSetSnapshot`
  - typed errors via `ValidatorLifecycleError`.
- Added integration and regression tests in `crates/kamn-core/tests/validator_lifecycle.rs`.

## Transition Proof and Validation Rules
- Transition proofs require:
  - non-empty `proposal_id`.
  - non-empty `proof_hash`.
  - non-empty approver DID list.
  - valid DID format for each approver.
- Proof approvals must satisfy the current quorum threshold.
- Duplicate validator DIDs are rejected.
- Transition proof fingerprint (`proposal_id` + `proof_hash`) is one-time-use and replay attempts are rejected.
- Onboarding proof approver sets cannot include the candidate validator DID (self-approval rejection).
- Regression guards:
  - transition proof replay is rejected (`Regression: #523`).
  - onboarding self-approval is rejected (`Regression: #523`).

## Quorum Safety and Rollback Rules
- Quorum threshold must be in `1..=validator_count`.
- Offboarding is blocked when resulting validator count would fall below current quorum threshold.
- Quorum reconfiguration is validated against current validator count.
- Rollback restores the previous snapshot and appends a rollback transition record.

## Governance Stake/Slash Threshold Gate Integration (Issue #750)
Validator lifecycle governance activation requires stake/slash risk evidence before applying quorum-impacting transitions.

- PR fast contract lane:
  - `bash scripts/governance/run_stake_slash_risk_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/governance/run_stake_slash_risk_deep_lane.sh --output-json governance-stake-slash-report.json`
- Required policy evidence:
  - stake-at-risk, slash-probability, and validator-churn thresholds remain within configured limits.
  - quorum safety margin remains above minimum threshold before execution.
  - tampered or incomplete risk evidence fails closed (`Regression: #733`).

## Governance Quorum Attestation Replay-Guard Integration (Issue #911)
Sensitive governance approvals require deterministic quorum attestation evidence and replay-resilient approval validation before execution.

- PR fast contract lane:
  - `bash scripts/governance/run_quorum_attestation_replay_guard_lane.sh --output-file /tmp/governance-quorum-attestation-replay-report.json`
- Policy checker:
  - `bash scripts/governance/check_quorum_attestation_replay_policy.sh --report-file /tmp/governance-quorum-attestation-replay-report.json`
- Stable shell wrapper:
  - `scripts/governance/check_quorum_attestation_replay_policy.sh`
- Shared Python implementation:
  - `scripts/governance/governance_quorum_attestation_replay_policy_contract.py`
- Contract lane:
  - `bash scripts/governance/run_quorum_attestation_replay_contract_lane.sh --output-file /tmp/governance-quorum-attestation-replay-contract-report.json`
- Required schema and reason-key markers:
  - `kamn.governance.quorum-attestation-replay-report.v1`
  - `governance_quorum_attestation_reason_codes:GO:v1`
  - `governance_quorum_attestation_reason_codes:NO-GO:v1`
- Required policy evidence:
  - required attestation keys remain present (`proposal_id`, `approval_artifact_id`, `payload_hash`, `approver_dids`).
  - signature metadata uses a supported algorithm with non-empty key id and positive signed-at timestamp.
  - received approval signatures satisfy required quorum threshold.
  - replayed approval artifacts force `NO-GO`.
  - quorum attestation evidence drift and replay attempts must fail closed (`Regression: #911`).

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test validator_lifecycle --test validator_lifecycle_docs
cargo test -p kamn-core --test governance_workflow governance_workflow_regression_rejects_replayed_voter_approval_artifact -- --exact
bash scripts/governance/test_run_quorum_attestation_replay_guard_lane.sh
bash scripts/governance/test_check_quorum_attestation_replay_policy.sh
bash scripts/governance/test_run_quorum_attestation_replay_contract_lane.sh
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
