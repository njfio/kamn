# kamn-cli Architecture

## Intent
Defines the architecture contract for `kamn-cli` in the KAMN workspace and documents its responsibilities/boundaries.

## Responsibilities
- `commands`

## Boundaries
- Owns crate-local behavior and contracts for `kamn-cli`.
- Depends on other workspace crates only through explicit Rust interfaces.
- Exposes stable surfaces expected by higher-level crates/workflows.

## Operational Notes
- Primary validation path: `cargo test -p kamn-cli`.
- Contract updates should be reflected in crate README and issue-local specs.

## Related
- `crates/kamn-cli/README.md`
- `docs/architecture/README.md`
