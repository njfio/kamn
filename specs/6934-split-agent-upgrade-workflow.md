# 6934-split-agent-upgrade-workflow

## Objective
Split `crates/kamn-core/src/agent_upgrade_workflow.rs` into bounded, concern-based modules while preserving proposal submission, human review promotion, governance submission, activation delay handling, audit event emission, and all existing error semantics.

## Inputs/Outputs
- Inputs:
  - workflow configuration payloads
  - agent upgrade proposal drafts
  - human review approvals
  - governance votes and execution inputs
  - activation timestamps and executor DIDs
- Outputs:
  - unchanged agent-driven upgrade workflow behavior
  - a thin root shell in `agent_upgrade_workflow.rs`
  - bounded sibling modules for config/models, workflow engine, validation/support, governance/orchestrator mapping, audit events, and tests
  - a hard-fail extraction contract for root shell and module layout

## Boundaries/Non-goals
- No changes to public reason codes, audit-event meanings, or governance/orchestration semantics
- No new dependencies
- No unrelated governance or upgrade orchestration refactors outside this workflow surface
- No behavior changes to proposal state transitions, quorum requirements, or activation delay rules

## Failure modes
- invalid config invariants remain fail-closed
- invalid proposer, reviewer, validator, or executor DIDs remain fail-closed
- duplicate proposal IDs remain fail-closed
- unauthorized proposer/reviewer/validator/executor paths remain fail-closed
- invalid timestamps and deadline ordering remain fail-closed
- extraction contract fails if root shell or module layout regress

## Acceptance criteria
- [ ] `crates/kamn-core/src/agent_upgrade_workflow.rs` becomes a thin root shell under the active file-size budget
- [ ] workflow concerns are split into bounded modules with clear responsibilities
- [ ] a hard-fail extraction contract enforces the root shell and module layout
- [ ] existing agent-upgrade workflow tests remain green without semantic drift
- [ ] touched-Rust size policy returns `policy_decision=GO`
- [ ] final spec records test evidence and any deviations

## Files to touch
- `crates/kamn-core/src/agent_upgrade_workflow.rs`
- `crates/kamn-core/src/agent_upgrade_workflow/`
- `crates/kamn-core/tests/agent_upgrade_workflow_module_extraction_contract.rs`
- `specs/6934-split-agent-upgrade-workflow.md`

## Error semantics
- Preserve existing typed `AgentUpgradeWorkflowError` behavior and all stable reason markers
- Preserve hard-fail validation for config, DID parsing, authorization, and timestamp ordering
- Do not introduce silent fallback or relaxed governance/activation behavior

## Test plan
- Add a red extraction contract that fails while `agent_upgrade_workflow.rs` is still monolithic
- Run the extraction contract green once the split is in place
- Run the real agent-upgrade workflow tests after extraction
- Run touched-Rust size policy against the staged write set
