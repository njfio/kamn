# kamn-snapshot-journal Architecture

## Intent
Defines the architecture contract for `kamn-snapshot-journal` in the KAMN workspace and documents its responsibilities/boundaries.

## Responsibilities
- (See crate source for internal modules.)

## Boundaries
- Owns crate-local behavior and contracts for `kamn-snapshot-journal`.
- Depends on other workspace crates only through explicit Rust interfaces.
- Exposes stable surfaces expected by higher-level crates/workflows.

## Operational Notes
- Primary validation path: `cargo test -p kamn-snapshot-journal`.
- Contract updates should be reflected in crate README and issue-local specs.

## Related
- `crates/kamn-snapshot-journal/README.md`
- `docs/architecture/README.md`
