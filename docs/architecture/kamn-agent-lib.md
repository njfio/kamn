# kamn-agent-lib Architecture

## Intent
Defines the architecture contract for `kamn-agent-lib` in the KAMN workspace and documents its responsibilities/boundaries.

## Responsibilities
- `auth`
- `client`
- `envelope`
- `errors`
- `identity`
- `kolme`
- `nonce`

## Boundaries
- Owns crate-local behavior and contracts for `kamn-agent-lib`.
- Depends on other workspace crates only through explicit Rust interfaces.
- Exposes stable surfaces expected by higher-level crates/workflows.

## Operational Notes
- Primary validation path: `cargo test -p kamn-agent-lib`.
- Contract updates should be reflected in crate README and issue-local specs.

## Related
- `crates/kamn-agent-lib/README.md`
- `docs/architecture/README.md`
