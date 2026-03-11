# kamn-types Architecture

## Intent
Defines the architecture contract for `kamn-types` in the KAMN workspace and documents its responsibilities/boundaries.

## Identity Markers
- `kamn_types_identity_boundary=did-helpers`
- `kamn_types_primary_module=kamn_types::did`
- `kamn_types_import_ownership=explicit`
- `kamn_types_current_dependency_status=owned-did-surface`
- `kamn_types_target_dependency_policy=no-kamn-core`
- `kamn_types_inversion_first_wave_csv=AgentDid,KamnDid,DidDocument,DidService,DidVerificationMethod`

## Responsibilities
- Own the canonical DID helper boundary (`kamn_types::did`).
- Re-export stable DID primitives and parse helpers for compatibility.
- Keep parse error semantics typed and fail-closed.

## Boundaries
- Owns crate-local behavior and contracts for `kamn-types`.
- Current state: owns the DID value surface directly and does not depend on `kamn-core`.
- Target state: preserve this leaf boundary and keep non-DID runtime behavior out of `kamn-types`.
- Exposes stable DID surfaces expected by higher-level crates/workflows.
- Non-DID runtime behavior stays outside this crate.

## Inversion Plan
- First wave moved reusable DID value types, parse errors, and canonical parse helpers from `kamn-core::did` into `kamn-types`.
- `kamn-core` consumes `kamn-types` and preserves temporary compatibility re-export shims during migration.
- Runtime registry/orchestration flows remain in `kamn-core` until follow-up extraction issues land.

## Operational Notes
- Primary validation path: `cargo test -p kamn-types`.
- Contract updates should be reflected in crate README and issue-local specs.

## Related
- `crates/kamn-types/README.md`
- `docs/architecture/did-format-standardization.md`
- `docs/architecture/README.md`
