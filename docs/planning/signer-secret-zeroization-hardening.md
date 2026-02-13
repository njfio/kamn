# Signer Secret Zeroization Hardening Notes

## Scope
- Runtime signer private-key parsing path in `crates/kamn-node/src/signer.rs`.
- Env-loaded private key hex buffers and decoded private key byte buffers.

## Hardening Decisions
- Zeroize env-loaded private key hex strings after parse attempts (success and failure paths).
- Zeroize decoded private key byte vectors immediately after `SigningKey` construction attempt.
- Zeroize partially decoded hex buffers on decode errors to reduce secret residue.
- Keep managed-external signer protocol flow unchanged; this change is scoped to env-local private-key adapter setup.

## Contracts
- Regression guard requires explicit zeroization markers in signer source:
  - `cargo test -p kamn-node main_tests::regression_signer_private_key_parse_path_requires_zeroize_markers -- --exact`
- Direct signer lifecycle tests:
  - `cargo test -p kamn-node signer::tests::unit_signer_private_key_parse_zeroizes_hex_buffer_on_success -- --exact`
  - `cargo test -p kamn-node signer::tests::regression_signer_private_key_parse_zeroizes_hex_buffer_on_failure -- --exact`
  - `cargo test -p kamn-node signer::tests::performance_signer_private_key_parse_zeroization_stays_bounded -- --exact`
