# Plan: Issue #6231 - Begin kamn-core Extraction Wave 1

## Approach

1. Add RED structural contract tests asserting expected extraction wave-1 façade/wiring.
2. Add new crates and move selected modules:
   - `kamn-crypto/src/direct_message_crypto.rs`
   - `kamn-bridges/src/cross_chain_receipt.rs`
   - `kamn-data-layer/src/data_layer_hashing.rs`
3. Replace corresponding `kamn-core` module files with façade re-exports.
4. Register new crates in workspace and add `kamn-core` path dependencies.
5. Run focused tests/build checks for extracted crates and `kamn-core`.

## Affected Modules

- `Cargo.toml` (workspace members)
- `crates/kamn-core/Cargo.toml`
- `crates/kamn-core/src/{direct_message_crypto.rs,cross_chain_receipt.rs,data_layer_hashing.rs}`
- new crates under `crates/kamn-{crypto,bridges,data-layer}/`
- `crates/kamn-core/tests/` (new extraction wave structural contract test)

## Risks and Mitigations

- Risk: test breakage from env-lock assumptions in moved crypto tests.
  - Mitigation: provide local test env lock in extracted crate tests.
- Risk: accidental API drift in moved modules.
  - Mitigation: façade re-export and targeted regression assertions.
- Risk: dependency/cycle issues.
  - Mitigation: keep moved modules self-contained in wave 1.

## Verification

- `cargo fmt --all --check`
- `cargo test -p kamn-core --test core_extraction_wave1_contract -- --nocapture`
- `cargo test -p kamn-crypto --lib -- --nocapture`
- `cargo test -p kamn-bridges --lib -- --nocapture`
- `cargo test -p kamn-data-layer --lib -- --nocapture`
- `cargo test -p kamn-core --test readme_contract_lane -- --nocapture`
