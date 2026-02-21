# R50.4 Milestone - Live-Postgres Selector Row-Format Contract Hardening

## Context
R50.3 introduced runtime selector-bundle integrity checks for duplicates, prefix, and row-count parity. This milestone tightens contracts by enforcing explicit row-format and canonical row-id validation.

## Scope
- Validate selector row format (`row_id->selector_path`).
- Validate canonical row-id membership.
- Extend deterministic validation test matrix.

## Deliverables
- Issue #5477 selector row-format and row-id contract hardening.

## Exit Criteria
- Runtime validation rejects malformed row format and non-canonical row IDs.
- Deterministic test matrix covers success and failure reason codes.
- Issue #5477 merged and spec status set to Implemented.
