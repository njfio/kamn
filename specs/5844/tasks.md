# Tasks: Issue #5844

## Ordered Tasks
- [x] T1 (RED): add conformance/regression tests proving deterministic baseline-v1 signatures are rejected by TCP envelope verification and forged handshake signatures fail closed.
- [x] T2 (GREEN): implement cryptographic TCP signature generation and verification using `kamn-core` service-auth secp256k1 helpers.
- [x] T3: extend TCP envelope and handshake wire payloads with signer public key metadata and enforce parity checks.
- [x] T4: update TCP tests, fixture expectations, and examples for the new wire/signature contract.
- [x] T5 (VERIFY): run scoped and full `kamn-sdk` tests and confirm replay behavior contract remains unchanged.

## Tier Mapping
- Unit: envelope parse/verify rejects deterministic signatures and malformed signer metadata.
- Functional: signed envelope roundtrip send/listen passes with cryptographic signatures.
- Conformance: C-01..C-06 mapped via `tcp_transport_adapter` and `tcp_failover_matrix`.
- Integration: reconnect/replay guard behavior remains unchanged under crypto signatures.
- Regression: forged handshake signature and tampered body mismatch fail closed.
