# kamn-types

## Purpose
Shared canonical type surface for cross-crate KAMN DID identifiers and parse helpers.

## Identity Boundary
- `kamn_types_identity_boundary=did-helpers`
- `kamn_types_primary_module=kamn_types::did`
- `kamn_types_import_ownership=explicit`

`kamn-types` owns canonical DID-facing helper behavior and typed parse errors.
Core runtime/storage behavior remains in `kamn-core`.

## Key Surfaces
- Primary imports:
  - `use kamn_types::did::AgentDid`
  - `use kamn_types::did::KamnDid`
  - `use kamn_types::did::parse_agent_did_canonical`
  - `use kamn_types::did::parse_kamn_did_canonical`
- Compatibility imports remain stable:
  - `use kamn_types::AgentDid`
  - `use kamn_types::KamnDid`
  - `use kamn_types::parse_agent_did_canonical`
  - `use kamn_types::parse_kamn_did_canonical`

## Migration Guidance
- `kamn_types_migration_import=use kamn_types::did::AgentDid`
- Prefer `kamn_types::did::*` for new or refactored call sites.
- Existing top-level `kamn_types::*` imports are preserved for non-breaking migration.

## Usage
- Build: `cargo build -p kamn-types`
- Test: `cargo test -p kamn-types`

## Notes
This README is part of the repository-wide crate documentation baseline tracked under issue #6260.
