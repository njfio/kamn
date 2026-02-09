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

## Escrow-Ledger Reconciliation Evidence Contract (Issue #717)
Escrow settlement accounting must include deterministic ledger-reference evidence before reconciliation gates can approve release.

- Evidence bundle generator:
  - `bash scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh --output-file /tmp/settlement-evidence.json --escrow-id escrow-001 --settlement-outcome RELEASED --receipt-id receipt-001 --receipt-finality FINAL --expected-release-amount 120 --expected-refund-amount 0 --observed-release-amount 120 --observed-refund-amount 0 --ledger-reference-id ledger-entry-001 --timeout-elapsed false --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/escrow/check_settlement_reconciliation_evidence_policy.sh --bundle-file /tmp/settlement-evidence.json`
- PR fast contract lane:
  - `bash scripts/escrow/run_settlement_reconciliation_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/escrow/run_settlement_reconciliation_deep_lane.sh --output-json settlement-reconciliation-report.json`
- Regression policy:
  - missing ledger reference evidence and ledger amount drift force `NO-GO` (`Regression: #717`).

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test audit_export_interfaces
cargo test -p kamn-core --test audit_export_interfaces_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```

## Follow-up
- Add explicit export scope permissions mapped to operator roles and tenant boundaries.
- Add stream-oriented export interfaces for large historical datasets.
