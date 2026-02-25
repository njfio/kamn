# Spec: Issue #6000 - Replace content-storage FNV CIDs with SHA-256 integrity

- Issue: #6000
- Status: Reviewed
- Type: story
- Priority: P0
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: #5917

## Problem Statement
`crates/kamn-core/src/content_storage.rs` currently derives CIDs/integrity tags from 64-bit FNV-1a (`kamn:cid:v1:<16-hex>` and `fnv1a64:<hex>`), which is non-cryptographic and collision-prone for integrity checks.

## Scope
In scope:
- Introduce SHA-256-backed CID and integrity tag generation for new writes.
- Add compatibility parsing/verification for legacy `kamn:cid:v1` + `fnv1a64:` records.
- Keep public content URI/CID APIs deterministic and backward-compatible.
- Add unit + integration + regression tests for cryptographic verification and legacy compatibility.

Out of scope:
- External database migration tooling.
- Changes to API route shapes.
- New storage engines.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: New content writes produce SHA-256-based CID and integrity tags (no FNV-only integrity for new data).
- AC-2: `verify()` cryptographically validates payload/tag integrity and fails on tampering.
- AC-3: Legacy `kamn:cid:v1` records remain readable and verifiable via explicit compatibility path.
- AC-4: URI conversion and CID validation continue to enforce strict format contracts for both legacy and new CID forms.

## Conformance Cases
- C-01 (Unit, AC-1): `put()` generates `kamn:cid:v2:<64-hex>` and `sha256:<64-hex>` for new records.
- C-02 (Functional, AC-2): payload or integrity-tag tampering causes `ContentStorageError::IntegrityMismatch`.
- C-03 (Integration, AC-3): serialized legacy v1 object set can still be loaded, read, and verified.
- C-04 (Regression, AC-4): `content_uri_for_cid` / `cid_from_content_uri` roundtrip succeeds for v1 and v2; invalid prefixes/hashes remain rejected.

## Success Metrics / Observable Signals
- New content integrity evidence is cryptographic (SHA-256) and collision-resistant.
- Existing persisted v1 content remains operational after upgrade.
- All mapped conformance tests pass.
