# Plan: Issue #6089

## Approach
1. Baseline the duplicate family inventory and shell LOC metrics before change.
2. Execute RED step by adding/using parity assertions that fail if migrated wrappers still embed full dispatcher logic.
3. Replace the duplicate family with thin wrappers that delegate to `run_non_kolme_contract_lane_dispatch.sh` using wrapper basename.
4. Run stale-reference and wrapper behavior checks for the migrated family.
5. Capture actual shell/rust ratio delta markers for PR/closure.

## Affected Modules
- `scripts/**/run_*_contract_lane.sh` (bounded duplicate family; exact file list materialized during implementation)
- `scripts/framework/run_non_kolme_contract_lane_dispatch.sh` (reference target only)
- `specs/6089/spec.md`
- `specs/6089/plan.md`
- `specs/6089/tasks.md`
- `specs/milestones/r67-runtime-hardening-and-surface-reduction/index.md`

## Risks / Mitigations
- Risk: wrapper semantics drift after dedup and break manifest resolution.
  Mitigation: parity check lanes run against representative wrappers and manifest resolution paths.
- Risk: stale references to removed/renamed paths.
  Mitigation: do not remove wrapper filenames; preserve command surface with compatibility stubs.
- Risk: lane-order contract with #6088.
  Mitigation: hold implementation until #6088 is merged; complete spec artifacts first.

## Interfaces / Contracts
- Command surface contract: wrapper paths remain invokable by existing callers.
- Dispatch contract: wrappers continue routing through `run_non_kolme_contract_lane_dispatch.sh` with wrapper-specific manifest resolution.
- No API/protocol/schema contract changes.
