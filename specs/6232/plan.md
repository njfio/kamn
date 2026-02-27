# Plan: Issue #6232 - Compact README and Move Contract References to Docs

## Approach

1. Add a RED docs contract test that enforces README <=200 lines and required onboarding anchors.
2. Introduce a dedicated docs reference page for long-form contract markers/commands.
3. Rewrite README to concise sections: overview, quickstart, architecture map, contributor guide, and links.
4. Update marker-coupled tests/scripts to reference the docs page where applicable.
5. Run targeted docs/contract suites for README and touched contract surfaces.

## Affected Modules

- `README.md`
- `docs/**` (new compact-reference page)
- `crates/kamn-core/tests/**` and/or `scripts/**` marker checks referencing README

## Risks and Mitigations

- Risk: Hidden marker couplings fail CI.
  - Mitigation: enumerate and migrate explicit marker checks; run targeted marker tests before PR.
- Risk: README loses essential onboarding flow.
  - Mitigation: enforce onboarding anchors with the new contract test.

## Verification

- `cargo fmt --all --check`
- Targeted docs contracts (README + touched marker suites)
- `cargo test -p kamn-core --test readme_compact_contract`
