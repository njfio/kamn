# Tasks: Issue #6217 - Fuzz Target for Kolme Flat-JSON Parser Surfaces

- [x] T1 (RED): extend cargo-fuzz contract tests to require the new target/corpus markers.
- [x] T2 (GREEN): add `kolme_flat_json_parser` target + dependency wiring + seed corpus.
- [x] T3 (REFACTOR): update replay metadata for deterministic target inventory.
- [x] T4 (VERIFY): run fmt, clippy target checks, and `cargo_fuzz_target_contract`.
