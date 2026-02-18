# Issue #5016 Spec

- Title: Task: M0 deliver core schema, append-only controls, and envelope crypto primitives
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD M0 requires a foundation data path that can deterministically persist encrypted message envelopes,
compute stable content hashes, and enforce append-only tamper checks before Merkle anchoring. This issue
delivers a Rust-native in-memory foundation slice in `kamn-core` so higher milestones can integrate on a
tested contract without increasing shell LOC.

## Acceptance Criteria
- AC-1: A deterministic envelope storage record can be derived from canonical envelope + ciphertext metadata with
  stable content hash and AAD hash outputs independent of map/list ordering.
- AC-2: Append-only controls reject duplicate message identifiers and expose no mutation API for existing entries.
- AC-3: Hash-chain verification detects tampering in stored records and succeeds for untampered append order.
- AC-4: Compression metadata constraints are enforced for M0 records (`compression_codec == \"zstd\"`,
  `compressed_size_bytes > 0`, `compressed_size_bytes <= content_size_bytes`).
- AC-5: Shell-surface impact remains neutral (`shell_loc_delta_actual = 0`) for this issue.

## Scope
In scope:
- New `kamn-core` module for M0 envelope record derivation and append-only ledger verification.
- Public API exports for integration by follow-on M1+ tasks.
- Deterministic unit/conformance tests mapped to C-01..C-05.

Out of scope:
- PostgreSQL migrations/triggers (tracked by follow-on tasks).
- Real cryptographic primitive swap-in requiring new dependency introduction.
- Merkle anchoring worker implementation.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Same envelope/ciphertext with reordered recipients and wrapped-key map | `content_hash` and `envelope_aad_hash` remain identical |
| C-02 | AC-2 | Functional | Append two records with duplicate `message_id` | Second append fails with duplicate-id error |
| C-03 | AC-3 | Regression | Verify untampered chain then tamper one record hash | First verify passes, second verify returns chain mismatch |
| C-04 | AC-4 | Unit | Construct record with invalid compression metadata | Constructor returns compression metadata error |
| C-05 | AC-5 | Regression | Compare issue diff shell/python/workflow LOC | Net shell delta remains zero |

## Test Mapping
- `cargo test -p kamn-core data_layer_m0` (unit + conformance for module)
- `cargo test -p kamn-core spec_c0` (conformance case selectors)
- `cargo test -p kamn-core` (regression sanity for crate before PR)
- Shell guard scripts are not required for this issue because shell surface is unchanged.

## Success Metrics
- All ACs map to passing `spec_c0x_*` tests.
- `kamn-core` remains green with no new clippy warnings for touched code.
- Shell LOC delta for this issue remains zero.
