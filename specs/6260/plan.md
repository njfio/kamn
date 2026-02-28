# Issue 6260 Plan

## Approach
1. Enumerate all crate directories and generate concise `README.md` files with consistent structure:
   - Purpose
   - Key modules/surfaces
   - Testing notes
2. Add seven architecture docs for crates lacking architecture coverage.
3. Update architecture navigation index with direct links.
4. Run documentation contract checks used in repository CI where available.

## Affected Paths
- `crates/*/README.md` (new files)
- `docs/architecture/*.md` (7 new files)
- `docs/architecture/README.md` (index updates)

## Risks and Mitigations
- Risk: boilerplate docs with insufficient specificity.
  - Mitigation: include crate-specific module surfaces from each crate's public API.
- Risk: navigation drift.
  - Mitigation: central index updated in same change.

## Interface/Contract Notes
- No runtime behavior changes.
- Documentation-only delivery.
