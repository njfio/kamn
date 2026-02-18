# Issue #5076 Plan

- Issue: #5076
- Status: Implemented

## Approach
1. Add M4 interop bridge contract types/functions:
   - fallible conversion from `EscrowStatus` -> `DataLayerM4EscrowState`.
   - explicit interop error taxonomy for ambiguous/unsupported legacy states.
2. Add M8 interop bridge contract types/functions:
   - mapping from `DataLayerM8RetentionClass` -> `Option<ContentRetentionClass>`.
   - fallible conversion from `DataLayerM8RetentionClass` -> `ContentRetentionClass`.
   - canonical conversion from `ContentRetentionClass` -> `DataLayerM8RetentionClass`.
3. Add explicit M8-vs-legacy retention-window alignment helper so drift is
   detectable and deterministic without changing M8 runtime retention semantics.
4. Extend existing M4 and M8 conformance test suites with bridge cases C-01..C-05.
5. Run scoped and crate-level regression gates.

## Risks and Mitigations
- Risk level: high
- Risks:
  - Incorrect state mapping could hide semantic drift.
  - Overly permissive retention mapping could break legal-hold/permanent semantics.
- Mitigations:
  - Keep mappings fail-closed for ambiguous/non-representable states.
  - Preserve existing M8 legal-hold/permanent behavior by returning `None`/error for legacy mapping.
  - Validate alignment against existing conformance suites.

## Interface Contract
- Additive public API only.
- No dependency, protocol, or wire-format changes.

## ADR
- Not required for this scoped additive interop integration.
