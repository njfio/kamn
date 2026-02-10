# Governance Proposal, Vote, and Execution Workflows (Issues #196 / #197 / #476)

This document captures the first implementation slice for protocol governance message workflows.

## Scope Delivered
- Added `crates/kamn-core/src/governance_workflow.rs` with:
  - `GovernanceWorkflow` for proposal submission, vote casting, status evaluation, and execution recording.
  - proposal/vote/execution models:
    - `GovernanceProposalDraft`
    - `GovernanceParameterChangeDraft`
    - `GovernanceProposalRecord`
    - `GovernanceVoteRecord`
    - `GovernanceExecutionRecord`
  - lifecycle and decision enums:
    - `GovernanceProposalStatus`
    - `GovernanceVoteChoice`
  - typed errors via `GovernanceWorkflowError`.
- Added integration and regression tests in `crates/kamn-core/tests/governance_workflow.rs`.

## Proposal Lifecycle Rules
- Proposal submission requires:
  - non-empty id/title/description.
  - valid proposer DID.
  - `created_at_unix > 0`.
  - voting deadline strictly greater than creation timestamp.
  - quorum threshold greater than zero.
- Proposal statuses:
  - `Voting`
  - `Approved`
  - `Rejected`
  - `Executed`
  - `Expired`

## Parameter Proposal Validation Rules
- Optional `parameter_change` payload supports typed governance parameter updates.
- Parameter payload validation requires:
  - non-empty `key` and `target_version`.
  - semver-style target version (`major.minor.patch` numeric segments).
  - `min_value <= max_value`.
  - `proposed_value` within `[min_value, max_value]`.
- Parameter catalog and compatibility policy:
  - `listener.quorum`: allowed range `[1, 7]`, supported from `1.0.0`.
  - `approver.required_approvals`: allowed range `[1, 7]`, supported from `1.0.0`.
  - `watchdog.delivery_ratio_bps`: allowed range `[9000, 9999]`, supported from `1.1.0`.
- Unknown keys, range-policy violations, and unsupported target-version combinations are rejected before proposal registration (`Regression: #476`).

## Vote and Quorum Rules
- Vote casting requires:
  - existing proposal id.
  - valid voter DID.
  - unique voter per proposal (duplicate votes rejected).
  - cast timestamp within voting window.
- Deterministic outcome rules:
  - yes votes reaching quorum => `Approved`.
  - no votes reaching quorum => `Rejected`.
  - voting window elapsed without quorum => `Expired`.

## Execution and Audit Rules
- Execution requires:
  - proposal in `Approved` status.
  - valid executor DID.
  - non-empty operation hash.
  - positive execution timestamp.
- Execution appends an immutable execution history record.
- Late votes after deadline are rejected with `ProposalClosed` + `Expired` status.

## Proposal Simulation and Human-Veto Evidence Contract (Issue #748)
Governance activation must include deterministic simulation and veto/timelock evidence before automation can proceed.

- Stable shell wrappers:
  - `scripts/governance/generate_governance_simulation_evidence_bundle.sh`
  - `scripts/governance/check_governance_simulation_policy.sh`
- Shared Python implementation:
  - `scripts/governance/governance_simulation_contract.py`
- Evidence bundle generator:
  - `bash scripts/governance/generate_governance_simulation_evidence_bundle.sh --output-file /tmp/governance-simulation.json --proposal-id gov-proposal-activation-001 --simulation-hash sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa --simulation-complete true --veto-window-open false --veto-recorded false --timelock-expired true --required-approvals 2 --received-approvals 2 --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/governance/check_governance_simulation_policy.sh --bundle-file /tmp/governance-simulation.json`
- PR fast contract lane:
  - `bash scripts/governance/run_governance_simulation_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/governance/run_governance_simulation_deep_lane.sh --output-json governance-simulation-report.json`
- Replay matrix runner:
  - `python3 scripts/governance/run_governance_simulation_matrix.py --fixture fixtures/governance_simulation/veto_timelock_cases.json --output-json governance-simulation-report.json`
- Regression policy:
  - simulation/veto bypass attempts and tampered evidence bundles force `NO-GO` (`Regression: #733`).

## Stake/Slash Risk Threshold Evidence Contract (Issue #750)
Governance activation also requires deterministic stake/slash risk thresholds to block unsafe economic outcomes.

- Stable shell wrappers:
  - `scripts/governance/generate_stake_slash_risk_evidence_bundle.sh`
  - `scripts/governance/check_stake_slash_risk_policy.sh`
- Shared Python implementation:
  - `scripts/governance/stake_slash_risk_contract.py`
- Evidence bundle generator:
  - `bash scripts/governance/generate_stake_slash_risk_evidence_bundle.sh --output-file /tmp/stake-slash-risk.json --proposal-id gov-risk-001 --simulation-hash sha256:1111111111111111111111111111111111111111111111111111111111111111 --stake-at-risk-bps 120 --max-stake-at-risk-bps 300 --slash-probability-bps 40 --max-slash-probability-bps 150 --validator-churn-bps 60 --max-validator-churn-bps 180 --quorum-safety-margin-bps 220 --min-quorum-safety-margin-bps 150 --evidence-complete true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/governance/check_stake_slash_risk_policy.sh --bundle-file /tmp/stake-slash-risk.json`
- PR fast contract lane:
  - `bash scripts/governance/run_stake_slash_risk_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/governance/run_stake_slash_risk_deep_lane.sh --output-json governance-stake-slash-report.json`
- Replay matrix runner:
  - `python3 scripts/governance/run_stake_slash_risk_matrix.py --fixture fixtures/governance_stake_slash/risk_threshold_cases.json --output-json governance-stake-slash-report.json`
- Regression policy:
  - unsafe threshold bypass attempts and tampered risk evidence force `NO-GO` (`Regression: #733`).

## Governance Lifecycle and Rollback Integrity Contract Lane
Governance execution paths now include deterministic lifecycle/rollback integrity checks with fail-closed reason-code evidence.

- Lifecycle/rollback lane command:
  - `bash scripts/governance/run_governance_lifecycle_rollback_lane.sh --output-file /tmp/governance-lifecycle-rollback-report.json`
- Stable shell wrapper:
  - `scripts/governance/run_governance_lifecycle_rollback_lane.sh`
- Shared Python implementation:
  - `scripts/governance/governance_lifecycle_rollback_lane_contract.py`
- Lifecycle/rollback policy checker:
  - `bash scripts/governance/check_governance_lifecycle_rollback_policy.sh --report-file /tmp/governance-lifecycle-rollback-report.json`
- Stable shell wrapper:
  - `scripts/governance/check_governance_lifecycle_rollback_policy.sh`
- Shared Python implementation:
  - `scripts/governance/governance_lifecycle_rollback_policy_contract.py`
- Lifecycle/rollback contract lane:
  - `bash scripts/governance/run_governance_lifecycle_rollback_contract_lane.sh --output-file /tmp/governance-lifecycle-rollback-contract-report.json`
- Stable shell wrapper:
  - `scripts/governance/run_governance_lifecycle_rollback_contract_lane.sh`
- Shared Python implementation:
  - `scripts/governance/governance_lifecycle_rollback_contract_lane_contract.py`

Runtime budget controls:

- `KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_MAX_SECONDS`
- `KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_CONTRACT_MAX_SECONDS`
- `KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_SKIP_COMMANDS`

Required schema/reason markers:

- `kamn.governance.lifecycle-rollback-report.v1`
- `governance_lifecycle_rollback_reason_codes:GO:v1`
- `governance_lifecycle_rollback_reason_codes:NO-GO:v1`

Regression policy:

- illegal lifecycle transitions and rollback integrity drift must fail closed (`Regression: #910`).
- shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1246`).

## Governance Quorum Attestation Replay Contract Lane
Governance quorum attestation and replay protections include deterministic reason-code evidence with fail-closed policy checks.

- Quorum attestation replay lane command:
  - `bash scripts/governance/run_quorum_attestation_replay_guard_lane.sh --output-file /tmp/governance-quorum-attestation-replay-report.json`
- Stable shell wrapper:
  - `scripts/governance/run_quorum_attestation_replay_guard_lane.sh`
- Shared Python implementation:
  - `scripts/governance/governance_quorum_attestation_replay_lane_contract.py`
- Quorum attestation replay policy checker:
  - `bash scripts/governance/check_quorum_attestation_replay_policy.sh --report-file /tmp/governance-quorum-attestation-replay-report.json`
- Stable shell wrapper:
  - `scripts/governance/check_quorum_attestation_replay_policy.sh`
- Shared Python implementation:
  - `scripts/governance/governance_quorum_attestation_replay_policy_contract.py`
- Quorum attestation replay contract lane:
  - `bash scripts/governance/run_quorum_attestation_replay_contract_lane.sh --output-file /tmp/governance-quorum-attestation-replay-contract-report.json`
- Stable shell wrapper:
  - `scripts/governance/run_quorum_attestation_replay_contract_lane.sh`
- Shared Python implementation:
  - `scripts/governance/governance_quorum_attestation_replay_contract_lane_contract.py`

Runtime budget controls:

- `KAMN_GOVERNANCE_QUORUM_ATTESTATION_MAX_SECONDS`
- `KAMN_GOVERNANCE_QUORUM_ATTESTATION_CONTRACT_MAX_SECONDS`
- `KAMN_GOVERNANCE_QUORUM_ATTESTATION_SKIP_COMMANDS`

Required schema/reason markers:

- `kamn.governance.quorum-attestation-replay-report.v1`
- `governance_quorum_attestation_reason_codes:GO:v1`
- `governance_quorum_attestation_reason_codes:NO-GO:v1`

Regression policy:

- quorum attestation evidence drift and replay attempts must fail closed (`Regression: #911`).
- shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1254`).

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
bash scripts/governance/test_generate_governance_simulation_evidence_bundle.sh
bash scripts/governance/test_run_governance_simulation_contract_lane.sh
bash scripts/governance/test_run_governance_simulation_matrix.sh
bash scripts/governance/test_run_governance_simulation_deep_lane.sh
bash scripts/governance/test_generate_stake_slash_risk_evidence_bundle.sh
bash scripts/governance/test_run_stake_slash_risk_contract_lane.sh
bash scripts/governance/test_run_stake_slash_risk_matrix.sh
bash scripts/governance/test_run_stake_slash_risk_deep_lane.sh
bash scripts/governance/test_run_governance_lifecycle_rollback_lane.sh
bash scripts/governance/test_check_governance_lifecycle_rollback_policy.sh
bash scripts/governance/test_run_governance_lifecycle_rollback_contract_lane.sh
bash scripts/governance/test_run_quorum_attestation_replay_guard_lane.sh
bash scripts/governance/test_check_quorum_attestation_replay_policy.sh
bash scripts/governance/test_run_quorum_attestation_replay_contract_lane.sh
cargo test -p kamn-core --test governance_workflow --test governance_workflow_docs
cargo test -p kamn-core --test upgrade_orchestration
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
