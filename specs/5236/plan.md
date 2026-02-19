# Issue #5236 Plan

- Issue: #5236
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Approach
1. Reproduce the CI failure locally with the same two failing test targets.
2. Replace legacy listener fixture DID literals in block-pipeline tests with parser-valid typed DID values.
3. Keep scenario intent intact (commit success, fork-choice rejection, stale-candidate rejection, budget assertion).
4. Re-run targeted tests and shell-ratio guard to confirm full conformance.

## Affected Modules
- `crates/kamn-core/tests/block_pipeline.rs`
- `crates/kamn-core/tests/block_pipeline_transport_fed.rs`

## Risks and Mitigations
- Risk: fixture updates accidentally alter intended error-ordering assertions.
  - Mitigation: only change invalid fixture literals; retain assertion logic and expected error variants.
- Risk: partial fixture migration leaves hidden call paths red.
  - Mitigation: execute both failing test targets in one command to validate full suite behavior.

## Interfaces / Contracts
- Typed DID fixtures must satisfy `AgentDid::parse` contract.
- No production API changes; test-only fixture conformance correction.
