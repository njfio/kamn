# Issue #5117 Plan

- Issue: #5117
- Status: Implemented

## Approach
1. Add RED tests for canonical-equivalent owner DID authorization and non-equivalent denial in M10 projection.
2. Replace local M10 DID parser with canonical `KamnDid::parse` helper.
3. Canonicalize owner-scope comparison inputs before authorization decision.
4. Preserve existing error taxonomy/reason codes and deterministic behavior.
5. Run targeted M10 tests, full M10 test files, fmt, clippy, and shell guardrails.

## Risks and Mitigations
- Risk level: low
- Risks:
  - Canonicalization may alter how malformed DIDs are surfaced in errors.
  - Scope normalization could unintentionally widen authorization behavior.
- Mitigations:
  - Keep mapping to existing `ComplianceProjectionFailed` and owner-scope reason markers.
  - Add explicit non-equivalent denial test.

## Interface Contract
- No public API signature changes.
- Internal owner DID canonicalization in M10 authorization only.

## ADR
- Not required (localized correctness integration).
