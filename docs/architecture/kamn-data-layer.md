# kamn-data-layer Architecture

## Intent
Defines the architecture contract for `kamn-data-layer` in the KAMN workspace and documents its responsibilities/boundaries.

## Responsibilities
- `data_layer_hashing`
- `data_layer_m10_compliance_projection_bookkeeping`
- `data_layer_m10_partition_month_policy`
- `data_layer_m10_partition_registry_state_machine`

## Boundaries
- Owns crate-local behavior and contracts for `kamn-data-layer`.
- Depends on other workspace crates only through explicit Rust interfaces.
- Exposes stable surfaces expected by higher-level crates/workflows.
- Does not own M8 compliance projection, DID normalization, or M10 phase-6 orchestration runtime.

## Operational Notes
- Primary validation path: `cargo test -p kamn-data-layer`.
- Contract updates should be reflected in crate README and issue-local specs.

## Related
- `crates/kamn-data-layer/README.md`
- `docs/architecture/README.md`
