# Issue 6249 Tasks

- T1 (Red/Baseline): Inventory `kamn-core` shim/facade modules and capture current consumer import paths.
- T2 (Green/Core): Retire shim wrappers from primary `kamn-core` export wiring and replace with direct extracted-crate re-exports.
- T3 (Green/Consumers): Migrate workspace consumers to direct extracted crates for migrated surfaces.
- T4 (Green/Docs): Update follow-up doc + ADR with keep/remove decisions and compatibility timeline.
- T5 (Regression): Run scoped `cargo test` for affected crates and paths.
