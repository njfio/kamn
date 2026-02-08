# Processor HA Runtime Contracts (Issues #354 / #357 / #360 / #363)

This document captures processor high-availability runtime contract text for snapshot restore guards and construct-lock safety rules.

## Scope Delivered
- Added processor HA docs contract baseline for runtime snapshot restore safeguards.
- Added construct-lock safety rules for split-brain and stale-lease boundaries.
- Added low-cost validation lane commands for docs-focused PR checks.

## Snapshot Restore Rules
- Snapshot restore requires deterministic expected state version and expected state hash inputs.
- Snapshot payloads must preserve stable state lineage fields for restore decisions.
- snapshot version/hash mismatch restores are rejected.

## Construct Lock Rules
- Processor construct-lock ownership must enforce single active lease semantics.
- split-brain lock acquisition attempts are rejected.
- stale lease renewal attempts are rejected.

## Test Coverage Mapping
- Unit: N/A (docs-focused contract slice).
- Functional: docs section assertions for snapshot and lock rules.
- Integration: docs command mapping assertions for runtime docs test lane.
- Regression:
  - snapshot restore mismatch rejection (`Regression: #361`)
  - split-brain and stale-renew lock rejection (`Regression: #362`)

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-node --test runtime_processor_ha_docs
cargo test -p kamn-node --test node_runtime_cli_docs
```

Then run strict formatting/lint gates:

```bash
cargo fmt --check
cargo clippy -p kamn-node -- -D warnings
```
