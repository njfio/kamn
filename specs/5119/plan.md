# Issue #5119 Plan

- Issue: #5119
- Status: Implemented

## Approach
1. Add RED regression tests proving canonical-equivalent owner DID lookup/query behavior.
2. Replace M5 local owner DID validator with canonical `KamnDid::parse` helper.
3. Canonicalize owner DID keys in append and canonicalize lookup inputs in owner-scoped APIs.
4. Preserve existing error taxonomy/reason codes and deterministic ordering.
5. Run targeted/full M5 tests, fmt, clippy, and shell guardrails.

## Risks and Mitigations
- Risk level: low
- Risks:
  - Canonical keying could change owner identifiers surfaced in some errors.
  - Broad owner-key touch points could regress unrelated M5 paths.
- Mitigations:
  - Add targeted regression tests for retention/query canonical lookups.
  - Run full M5 test suite before PR.

## Interface Contract
- No public API signature changes.
- Internal canonicalization of owner DID keys/lookups.

## ADR
- Not required (localized correctness integration).
