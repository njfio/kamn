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

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test governance_workflow --test governance_workflow_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
