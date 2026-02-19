# Issue #5228 Plan

- Issue: #5228
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Approach
1. Introduce typed DID validated wrappers for wave-A boundary structs/inputs while preserving external API stability where practical.
2. Centralize DID parsing through helper functions that emit structured invalid-DID errors with deterministic reason codes.
3. Update bridge/marketplace module validation and routing paths to consume validated wrappers.
4. Update wave-A tests to assert deterministic invalid-DID reason mappings and preserve existing behavior contracts.
5. Run targeted wave-A suites plus shell-ratio guardrail.

## Affected Modules
- `bridge_adapter.rs`: validated inbound/outbound/requester bridge DID wrappers; deterministic invalid-DID reason mapping.
- `cross_chain_bridge.rs`: validated config/request wrappers for listener/approver/route target DIDs.
- `discord_bridge.rs`: validated config/request wrappers for listener/approver/route target DIDs.
- `telegram_bridge.rs`: validated config/request wrappers for listener/route target DIDs.
- `service_marketplace.rs`: validated provider/requester DID wrappers for listing registration and negotiation hooks.

## Risks and Mitigations
- Risk: Broad test fallout due boundary-shape drift.
  - Mitigation: keep public struct surfaces stable where possible and validate via `TryFrom<&Raw>`.
- Risk: nondeterministic error text checks.
  - Mitigation: assert reason-code markers rather than parser free-form text.
- Risk: shell-surface regression.
  - Mitigation: Rust-only edits; run shell-ratio guardrail check.

## Interfaces / Contracts
- Invalid DID errors carry:
  - `field`: boundary field name
  - `reason_code`: deterministic marker
  - `detail`: parser detail string
- Conversion entry points use `TryFrom<&RawType>` validated wrappers.
