# Issue #5121 Plan

- Issue: #5121
- Status: Reviewed

## Approach
1. Add RED tests for canonical-equivalent owner DID lookup/authorization in M8.
2. Replace local M8 owner DID parser with canonical `KamnDid::parse` helper.
3. Canonicalize owner keys on insert and canonicalize owner-scope comparisons/lookups.
4. Preserve existing error taxonomy/reason-code behavior.
5. Run targeted/full M8 tests, fmt, clippy, and shell guardrails.

## Risks and Mitigations
- Risk level: low
- Risks:
  - Canonicalization could affect error payload owner DID formatting.
  - Multiple M8 APIs share owner scope; partial conversion could introduce drift.
- Mitigations:
  - Convert all owner-scope entry points in one commit.
  - Use regression tests for lookup and authorization paths.

## Interface Contract
- No API signature changes.
- Internal owner DID canonicalization only.

## ADR
- Not required (localized correctness integration).
