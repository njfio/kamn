# Plan: Issue #6000

## Approach
1. Add SHA-256 digest helper usage in `content_storage` for CID and integrity derivation.
2. Introduce versioned CID support:
   - `kamn:cid:v2:<sha256-hex>` for new writes.
   - Compatibility read/verify path for legacy `kamn:cid:v1:<fnv-hex>`.
3. Update validation helpers to accept both v1 and v2 formats with strict hex-width checks.
4. Make `verify()` and deserialization path choose hash algorithm based on CID version.
5. Add RED/GREEN tests for v2 write behavior, tamper detection, and v1 compatibility.

## Affected Modules
- `crates/kamn-core/src/content_storage.rs`

## Risks / Mitigations
- Risk: Breaking existing persisted content with v1 CIDs.
  Mitigation: Explicit v1 compatibility checks and regression tests loading v1 fixtures.
- Risk: Partial migration confusion between integrity tag algorithms.
  Mitigation: Derive expected tag from CID version during verification.
- Risk: Hidden format regressions in URI helpers.
  Mitigation: Add roundtrip tests for both CID versions plus strict negative tests.

## Interfaces / Contracts
- Public API types unchanged.
- CID format expands from legacy-only v1 to versioned v1/v2 acceptance.
- New writes are always v2/SHA-256.
