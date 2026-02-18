# Issue #4970 Plan

- Issue: #4970
- Status: Implemented

## Approach
- Implement issue #4970 using Red -> Green -> Refactor -> Regression loop.
- Keep shell-surface and process-contract outputs deterministic and fail closed.
- Limit scope strictly to issue #4970 boundaries.

## Affected Modules
- `fixtures/ci/superseded_script_deletion_manifest.json`
- `scripts/ci/stale_script_reference_detector.py`
- `scripts/ci/test_check_stale_script_references.sh`
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_ci_tools_command_surface_contract.sh`
- `.ci/kolme-command-surface-asymmetry-policy.json`
- `scripts/kolme/run_contract_lane_dispatch.sh`
- `scripts/kolme/contract_lane_dispatch_impl.sh`
- `scripts/kolme/test_contract_lane_dispatch_wrapper_compaction.sh`

## Risks and Mitigations
- Risk level: high
- Mitigation: phase work in small verifiable commits, keep contract-lane checks green, and gate merges on deterministic test evidence.

## Interface Contract
- No protocol/wire-format changes without explicit approval.
- Reason taxonomy and marker outputs remain stable unless explicitly versioned.

## ADR
- Open ADR only if issue #4970 introduces architecture/dependency/protocol changes.
