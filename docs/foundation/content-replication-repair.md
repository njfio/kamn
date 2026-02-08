# Content Pinning, Replication, and Repair Policy Loops (Issues #166 / #167)

This document captures the first implementation slice for content availability policy and deterministic repair planning.

## Scope Delivered
- Added `crates/kamn-core/src/content_replication.rs` with:
  - `ContentReplicationPolicy` validation for minimum/target replicas and retry bounds.
  - `ContentReplicationManager` with deterministic tracking, repair planning, and repair result application.
  - `ContentAvailabilityHealth`, `ContentAvailabilitySnapshot`, and `ContentAvailabilityAlert`.
  - `ContentRepairAction` and `ContentRepairReason` for explicit repair work units.
  - typed errors via `ContentReplicationError`.
- Added integration and regression tests in `crates/kamn-core/tests/content_replication_repair.rs`.

## Availability and Repair Rules
- Health states:
  - `Healthy` when available replicas meet minimum threshold.
  - `Degraded` when replicas are non-zero but below minimum threshold.
  - `Unavailable` when replica count is zero.
- Repair planning:
  - plans repairs when available replicas are below target threshold.
  - uses deterministic CID ordering.
  - suppresses duplicate repair actions while a repair is pending.
- Retry safety:
  - repair failures increment per-content attempt counters.
  - repair planning stops after `max_repair_attempts`.

## Storage Integrity Integration
- `register_content(...)` validates content through storage adapter `head` and `verify`.
- Tampered storage content is rejected via `ContentReplicationError::Storage(...)`.
- Repair manager does not mutate payload bytes; it only tracks replica health metadata.

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test content_replication_repair --test content_replication_repair_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
