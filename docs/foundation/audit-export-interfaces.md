# Compliance Audit Export Interfaces Slice (Issues #152, #153)

This document describes the first implementation slice for multi-domain compliance audit exports.

## Scope Delivered
- Added `AuditExportEngine` in `kamn-core` with deterministic export behavior across domains:
  - `messages`
  - `tasks`
  - `escrows`
  - `reputation`
- Added structured export contract:
  - `AuditEventRecord`
  - `AuditExportRequest`
  - `AuditExportFilter`
  - `AuditExportBundle`
  - `AuditExportManifest`
- Enforced export access controls:
  - Only authorized exporter DIDs can generate exports.
- Implemented deterministic filtering and ordering:
  - Domain filter
  - Actor allowlist filter
  - Inclusive time window filter
- Added integrity metadata:
  - Canonical export payload serialization
  - Deterministic `fnv1a64` hash in manifest

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test audit_export_interfaces
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```

## Follow-up
- Add explicit export scope permissions mapped to operator roles and tenant boundaries.
- Add stream-oriented export interfaces for large historical datasets.
