# ADR-002: Runtime Guard Contracts Phase-1 Extraction From `kamn-core`

## Context

`kamn-core` has grown into a large multi-domain crate. Runtime guard contracts
(anti-spam, quota/fairness policy, delivery replay guards, retention, and
watchdog classification) are self-contained and stable enough to extract as a
first decomposition slice without wire-format or API changes.

Issue link: `#5933`
Spec link: `specs/5933/spec.md`

## Decision

Create a focused crate, `crates/kamn-runtime-guards`, and move runtime guard
contract implementations into it:

- `anti_spam`
- `fairness_policy`
- `quota_policy`
- `message_delivery_guards`
- `retention_engine`
- `watchdog`

Retain compatibility module paths in `kamn-core` by replacing each moved module
with a re-export shim (`pub use kamn_runtime_guards::<module>::*;`).

## Consequences

- `kamn-core` keeps stable public paths while compile ownership for runtime
  guards shifts into a focused crate.
- Runtime guard tests run independently with `cargo test -p kamn-runtime-guards`.
- Future decomposition can continue by extracting adjacent domains without
  immediate breaking API changes.
