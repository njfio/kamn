# Issue #3913 Tasks

- Issue: `#3913`
- Status: `InProgress`

## Ordered Tasks
- T1 (Regression): verify success/failure zeroization unit tests remain active.
- T2 (Conformance): add signer decode/source docs drift guard checks.
- T3 (Verify): run scoped signer and contract suites.

## Verification Commands
- `cargo test -p kamn-node signer::tests::unit_signer_private_key_parse_zeroizes_hex_buffer_on_success -- --exact --nocapture`
- `cargo test -p kamn-node signer::tests::regression_signer_private_key_parse_zeroizes_hex_buffer_on_failure -- --exact --nocapture`
- `cargo test -p kamn-node --test signer_secret_hygiene_contract -- --nocapture`

## Completion Evidence
- decode-path zeroization remains explicit and contract-guarded.
