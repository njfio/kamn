# Plan — #4375

## Approach
- Add a summary tamper case in local KAMN runtime integration contract-lane shell test.
- Assert checker fails closed and emits deterministic in-memory provider reason code.

## Risks
- Risk: Existing taxonomy assertions may require synchronized updates.
  - Mitigation: keep tamper case scoped to one marker and one expected reason.

## Interfaces
- Expected deterministic reason: `runtime_commit_in_memory_provider_reference_detected`.

## ADR
- Not required.
