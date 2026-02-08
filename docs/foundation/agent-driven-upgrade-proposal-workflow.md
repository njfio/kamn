# Agent-Driven Protocol Upgrade Proposal Workflow (Issues #234 / #235 / #528 / #533 / #538)

This document captures the first implementation slice for a pilot agent-driven protocol upgrade workflow with mandatory human and validator safeguards.

## Scope Delivered
- Added `crates/kamn-core/src/agent_upgrade_workflow.rs` with:
  - `AgentDrivenUpgradeWorkflow` orchestration across:
    - `submit_agent_proposal(...)`
    - `approve_human_review(...)`
    - `submit_to_governance(...)`
    - `cast_validator_vote(...)`
    - `finalize_upgrade(...)`
  - workflow models:
    - `AgentUpgradeWorkflowConfig`
    - `AgentUpgradeProposalDraft`
    - `AgentUpgradeProposalRecord`
    - `AgentUpgradeProposalState`
  - audit projection surfaces:
    - `AgentUpgradeAuditEvent`
    - `AgentUpgradeAuditEventKind`
  - typed errors via `AgentUpgradeWorkflowError`.
- Added integration and regression tests in `crates/kamn-core/tests/agent_upgrade_workflow.rs`.

## Workflow Safeguards
- Agent proposal submission requires:
  - allowlisted proposer DID.
  - non-empty proposal id and rationale.
  - valid timestamps with deadline strictly after creation.
- Validator governance voting requires:
  - allowlisted validator voter DID.
- Human review approvals require:
  - allowlisted validator reviewer DID.
- Governance submission requires:
  - sufficient unique human reviewer approvals.
  - pending-human-review proposal state.
- Upgrade finalization requires:
  - governance status `Approved`.
  - configured minimum activation delay elapsed since governance approval timestamp.
  - governance execution recording.
  - application of validator `Yes` votes as upgrade approvals before activation.

## Governance and Audit Rules
- Agent workflow emits deterministic audit events for:
  - agent proposal intake
  - human review approvals
  - governance submission/approval/execution
  - final upgrade activation
- Governance and version-upgrade records remain queryable through:
  - `governance_record(...)`
  - `proposal(...)`
  - `upgrade_audit_view(...)`
  - `agent_audit_log(...)`
- Regression guard:
  - early activation before required delay is rejected (`Regression: #528`).
  - unauthorized validator vote is rejected (`Regression: #533`).
  - unauthorized human reviewer approval is rejected (`Regression: #538`).

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test agent_upgrade_workflow --test agent_upgrade_workflow_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
