# Issue #5115 Plan

- Issue: #5115
- Status: Implemented

## Approach
1. Add RED regression tests capturing canonical-equivalent owner DID scope and lookup behavior.
2. Replace M6 local owner DID validator with `KamnDid::parse` helper.
3. Canonicalize owner DID values used for map keys and requester/owner equality checks.
4. Keep existing error taxonomy/reason codes stable while normalizing internal owner DID handling.
5. Run targeted and full M6 tests plus lint/format checks.

## Risks and Mitigations
- Risk level: low
- Risks:
  - Canonicalization could change owner key storage and break existing tests expecting raw whitespace.
  - Signature-preserving API paths returning `Option` may hide parse errors.
- Mitigations:
  - Add explicit regression tests for lookup/scope behavior.
  - Preserve public signatures and deterministic ordering.
  - Map parse failures to existing `InvalidDid` error variant where API returns `Result`.

## Interface Contract
- No public API signature changes.
- Internal normalization only for owner DID scope handling.

## ADR
- Not required (localized integration correction).
