# Tasks: Issue #5897 - Cryptographic Signer Migration for Core Signer Backend

1. T1 (RED/Conformance): add failing tests proving baseline-v1 signatures are rejected by default and tamper cases fail for cryptographic signatures.
2. T2 (Implementation): wire signer backend sign/verify to cryptographic secp256k1 helpers with explicit key-material resolution.
3. T3 (Implementation): add explicit legacy compatibility switch for baseline-v1 verification.
4. T4 (GREEN/Functional): run signer backend unit/integration suites and keep role/handshake/fallback behavior green.
5. T5 (Mutation): run in-diff mutation testing on touched signer verification path and achieve zero missed mutants.
6. T6 (Docs/Contracts): update affected signer-path contract docs/tests if behavior markers changed.
