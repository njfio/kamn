# kamn-data-layer Architecture

## Intent
Defines the architecture contract for `kamn-data-layer` in the KAMN workspace and documents its responsibilities/boundaries.

## Responsibilities
- `data_layer_hashing`
- `data_layer_m1_batch_scheduler`
- `data_layer_m7_billing_reconciliation`
- `data_layer_m7_observability_projection`
- `data_layer_m10_compliance_projection_bookkeeping`
- `data_layer_m10_partition_month_policy`
- `data_layer_m10_partition_registry_state_machine`
- `data_layer_m11_closure_evidence`
- `data_layer_m11_hardening_readiness`
- `data_layer_prd_critical_scenario_conformance`
- `data_layer_shell_neutral_policy`

## Boundaries
- Owns crate-local behavior and contracts for `kamn-data-layer`.
- Depends on other workspace crates only through explicit Rust interfaces.
- Exposes stable surfaces expected by higher-level crates/workflows.
- Does not own M8 compliance projection, DID normalization, or M10 phase-6
  orchestration runtime.

## Operational Notes
- Primary validation path: `cargo test -p kamn-data-layer`.
- Contract updates should be reflected in crate README and issue-local specs.

## Related
- `crates/kamn-data-layer/README.md`
- `docs/architecture/README.md`
