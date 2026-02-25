# ADR-001: Production Message Crypto Primitives

- Date: 2026-02-25
- Status: Accepted
- Related issue: #5921

## Context
Direct-message and group-channel crypto paths were implemented with deterministic XOR/FNV fixtures. That design did not provide production confidentiality or integrity guarantees.

The runtime contract requires canonical message encryption primitives:
- key agreement: `X25519`
- authenticated encryption: `XChaCha20-Poly1305`

## Decision
1. Adopt `x25519-dalek` for key agreement operations.
2. Adopt `chacha20poly1305` (XChaCha20-Poly1305) for authenticated encryption.
3. Adopt `sha2` for domain-separated key derivation and signature-token derivation.
4. Require `KAMN_KEY_AGREEMENT_MASTER_SEED_HEX` (32-byte hex) for key material derivation.
5. Remove insecure env-gated deterministic fixture crypto paths from constructors.

## Consequences
### Positive
- Direct and group message paths now use real cryptographic primitives.
- Tamper resistance is enforced by AEAD authentication, not ad-hoc hash tags.
- Constructors fail closed when key material provisioning is missing.

### Tradeoffs
- Adds crypto dependencies (`x25519-dalek`, `chacha20poly1305`, `sha2`) to `kamn-core`.
- Deployments must provision `KAMN_KEY_AGREEMENT_MASTER_SEED_HEX`.

### Follow-up
- Replace seed-derived key material with DID/key-store backed key resolution to remove shared-seed operational coupling.
