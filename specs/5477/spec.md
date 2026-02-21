# Issue #5477 Spec - Selector Row-Format and Row-ID Validation Hardening

- Status: Implemented
- Issue: #5477
- Parent: #3812
- Milestone: R50.4 Live-postgres selector row-format contract hardening

## Problem Statement
Runtime selector-bundle validation currently misses explicit checks for canonical `row_id->selector` formatting and canonical row-id membership, allowing malformed row strings to bypass intended strictness.

## Scope
In scope:
- Add row-format validation and canonical row-id validation to runtime selector-bundle checks.
- Extend selector-bundle validation contract tests with deterministic reason-code coverage.
- Preserve canonical-bundle success behavior.

Out of scope:
- New telemetry/report schema fields.
- New live-postgres runtime topology behavior.

## Acceptance Criteria
- AC-1: Validation rejects rows without `row_id->selector` format.
- AC-2: Validation rejects rows with non-canonical row IDs.
- AC-3: Validation test matrix deterministically covers valid + all failure reason codes (duplicate, prefix, row-count, format, row-id).

## Conformance Cases
- C-01 (Unit, AC-1): malformed row format returns `live_postgres_selector_bundle_row_format_violation`.
- C-02 (Unit, AC-2): non-canonical row-id returns `live_postgres_selector_bundle_row_id_violation`.
- C-03 (Conformance, AC-3): validation matrix test passes covering canonical and all deterministic failure reasons.

## Success Metrics / Observable Signals
- Runtime selector-bundle validation catches malformed contracts earlier.
- Deterministic reason code matrix is complete and enforced in tests.
- Existing daemon phase6 runtime marker tests remain green.
