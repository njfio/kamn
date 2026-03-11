# kamn-governance

## Purpose

`kamn-governance` holds the extracted governance workflow, operator binding, and permissioned operator action contracts that were previously embedded in `kamn-core`.

## Exported Surfaces

- `GovernanceWorkflow`: proposal submission, voting, evaluation, execution, and query APIs
- `GovernanceProposalDraft` / `GovernanceProposalRecord`: governance proposal input and state snapshots
- `GovernanceVoteChoice` / `GovernanceVoteRecord`: voting choices and recorded vote history
- `GovernanceExecutionRecord`: execution audit output for approved proposals
- `OperatorBindingEngine`: operator binding registration, authorization, and revocation
- `OperatorBindingAction` / `OperatorBindingRecord` / `OperatorBindingProof`: binding policy model surface
- `PermissionedOperatorActionService`: fail-closed operator configuration and audit service

## Local Verification

Run the crate test suite directly:

```bash
cargo test -p kamn-governance -- --nocapture
```

For targeted workflow coverage:

```bash
cargo test -p kamn-governance governance_workflow_internal -- --nocapture
```
