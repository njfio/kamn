# kamn-crypto Architecture

## Intent
Defines the architecture contract for `kamn-crypto` in the KAMN workspace and documents its responsibilities/boundaries.

## Responsibilities
- `direct_message_crypto`

## Boundaries
- Owns crate-local behavior and contracts for `kamn-crypto`.
- Depends on other workspace crates only through explicit Rust interfaces.
- Exposes stable surfaces expected by higher-level crates/workflows.

## Operational Notes
- Primary validation path: `cargo test -p kamn-crypto`.
- Contract updates should be reflected in crate README and issue-local specs.

## Related
- `crates/kamn-crypto/README.md`
- `docs/architecture/README.md`
