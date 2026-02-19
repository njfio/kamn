# Issue #3940 Tasks

- Issue: #3940
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add failing regression for source-extractor behavior with top-level `#[cfg(test)]` attributes.
- [x] T2 (Green): implement cfg(test)-item skipping extractor and extend runtime file coverage checks.
- [x] T3 (Docs): update runtime watchdog attestation panic-path retirement mapping for #3940.
- [x] T4 (Regression): run mapped tests for extractor, startup panic-path checks, and docs contract.
- [x] T5 (Verify): run `cargo fmt --check` and strict `cargo clippy -p kamn-node -- -D warnings`.

## Tier Mapping
- Unit: extractor fixture regression proving post-cfg production lines remain visible.
- Functional: runtime source list guard for production panic primitives.
- Integration: combined node panic-path + core docs contract checks.
- Regression: #3940 parser false-negative regression and docs mapping verification.
- Performance: N/A (test/docs hardening only; no runtime hot-path changes).
