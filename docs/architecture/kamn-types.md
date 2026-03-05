# kamn-types Architecture

## Intent
Defines the architecture contract for `kamn-types` in the KAMN workspace and documents its responsibilities/boundaries.

## Identity Markers
- `kamn_types_identity_boundary=did-helpers`
- `kamn_types_primary_module=kamn_types::did`
- `kamn_types_import_ownership=explicit`

## Responsibilities
- Own the canonical DID helper boundary (`kamn_types::did`).
- Re-export stable DID primitives and parse helpers for compatibility.
- Keep parse error semantics typed and fail-closed.

## Boundaries
- Owns crate-local behavior and contracts for `kamn-types`.
- Depends on `kamn-core` through explicit Rust interfaces only.
- Exposes stable DID surfaces expected by higher-level crates/workflows.
- Non-DID runtime behavior stays outside this crate.

## Operational Notes
- Primary validation path: `cargo test -p kamn-types`.
- Contract updates should be reflected in crate README and issue-local specs.

## Related
- `crates/kamn-types/README.md`
- `docs/architecture/README.md`
