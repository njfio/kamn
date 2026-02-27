# Issue 6227 Tasks

- T1 (Red/Regression): Add failing tests for legacy-v1 ciphertext decrypt compatibility in direct/group crypto modules.
- T2 (Green/Implementation): Implement HKDF-SHA256 derivation for direct/group AEAD keys.
- T3 (Green/Compatibility): Add v2-first + v1-fallback decrypt behavior without changing wire format.
- T4 (Regression): Run targeted crypto tests for direct/group modules in `kamn-core`.
- T5 (Verification): Map AC/C-cases to tests and finalize issue closure evidence.
