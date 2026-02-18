# Issue #4973 Plan

- Issue: #4973
- Status: Planned

## Approach
- Implement issue #4973 using Red -> Green -> Refactor -> Regression loop.
- Keep shell-surface and process-contract outputs deterministic and fail closed.
- Limit scope strictly to issue #4973 boundaries.

## Affected Modules
- To be finalized in implementation branch for #4973.

## Risks and Mitigations
- Risk level: high
- Mitigation: phase work in small verifiable commits, keep contract-lane checks green, and gate merges on deterministic test evidence.

## Interface Contract
- No protocol/wire-format changes without explicit approval.
- Reason taxonomy and marker outputs remain stable unless explicitly versioned.

## ADR
- Open ADR only if issue #4973 introduces architecture/dependency/protocol changes.
