# kamn-types Architecture

## Intent
Defines the architecture contract for `kamn-types` in the KAMN workspace and documents its responsibilities/boundaries.

## Responsibilities
- (See crate source for internal modules.)

## Boundaries
- Owns crate-local behavior and contracts for `kamn-types`.
- Depends on other workspace crates only through explicit Rust interfaces.
- Exposes stable surfaces expected by higher-level crates/workflows.

## Operational Notes
- Primary validation path: `cargo test -p kamn-types`.
- Contract updates should be reflected in crate README and issue-local specs.

## Related
- `crates/kamn-types/README.md`
- `docs/architecture/README.md`
