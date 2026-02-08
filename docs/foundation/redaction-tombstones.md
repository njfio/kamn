# Redaction and Tombstone Compliance Slice (Issues #150, #151)

This document describes the first implementation slice for PRD-aligned redaction and tombstone compliance workflows.

## Scope Delivered
- Added `RedactionComplianceEngine` in `kamn-core` for controlled redaction/tombstone workflows.
- Implemented quorum-based approval flow:
  - Requests remain pending until required approvals are collected.
  - Quorum automatically applies protection (redacted or tombstoned visibility).
- Implemented deterministic retrieval behavior:
  - `Available`
  - `Redacted { request_id }`
  - `Tombstoned { request_id }`
- Added immutable audit evidence stream per request:
  - `Requested`, `Approved`, `Rejected`, `Applied`
- Added target protection guard preventing silent restore/override for already protected targets.
- Integrated canonical state-key usage for target indexing.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test redaction_tombstones
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```

## Follow-up
- Add explicit requester/approver role policy checks and configurable authorization chains.
- Extend integration with storage and query APIs for operator-facing redaction history views.
