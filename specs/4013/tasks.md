# Issue #4013 Tasks

## Ordered Tasks

- [ ] T1 (RED): add failing checker tests in `crates/kamn-core/tests/cross_store_replay_consistency.rs`.
- [ ] T2 (Implement): add cross-store replay consistency checker module and deterministic taxonomy projection.
- [ ] T3 (Expose): export checker APIs via `crates/kamn-core/src/lib.rs`.
- [ ] T4 (Integration): add composed store->snapshot->checker integration test coverage.
- [ ] T5 (Docs Conformance): update `docs/foundation/runtime-network.md` and docs-contract assertions.
- [ ] T6 (GREEN Verify): run targeted checker/docs tests until green.
- [ ] T7 (Quality): run `cargo fmt --check` and `cargo clippy -p kamn-core --tests -- -D warnings`.

## Dependency Notes

- T2 depends on T1 RED evidence.
- T3 depends on T2.
- T4 depends on T2/T3.
- T5 depends on T2/T3 (final marker names).
- T6 depends on T2..T5.
- T7 runs after T6.
