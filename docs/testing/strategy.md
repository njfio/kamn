# Testing Strategy

## Suite Modularization Conventions

- Large integration suites should be organized into domain modules instead of a
  single monolithic file.
- Root integration harness files should declare explicit domain modules using
  `#[path = \"...\"] mod ...;` and avoid embedding all test logic inline.
- Shared fixtures/configuration helpers should live in a dedicated `shared.rs`
  module under the suite directory.

## Current Modularized Suites

- `task_escrow_proptest_invariants`
  - root harness: `crates/kamn-core/tests/task_escrow_proptest_invariants.rs`
  - domain modules:
    - `crates/kamn-core/tests/task_escrow_proptest_invariants/task_domain.rs`
    - `crates/kamn-core/tests/task_escrow_proptest_invariants/escrow_domain.rs`
  - shared module:
    - `crates/kamn-core/tests/task_escrow_proptest_invariants/shared.rs`
