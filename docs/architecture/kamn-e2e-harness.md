# kamn-e2e-harness Architecture

## Intent
Defines the architecture contract for `kamn-e2e-harness` in the KAMN workspace and documents its responsibilities/boundaries.

## Responsibilities
- `drivers`
- `evidence`
- `identity`
- `infrastructure`
- `kolme_devnet`
- `scenarios`
- `verify`

## Boundaries
- Owns crate-local behavior and contracts for `kamn-e2e-harness`.
- Depends on other workspace crates only through explicit Rust interfaces.
- Exposes stable surfaces expected by higher-level crates/workflows.

## Operational Notes
- Primary validation path: `cargo test -p kamn-e2e-harness`.
- Contract updates should be reflected in crate README and issue-local specs.

## Related
- `crates/kamn-e2e-harness/README.md`
- `docs/architecture/README.md`
