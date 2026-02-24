# Spec: Issue #5899 - Immediate Security/Runtime Remediation (Production Blockers)

- Issue: #5899
- Status: Accepted
- Type: task
- Priority: P0
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
Security-critical runtime paths still expose non-production-safe behavior: non-cryptographic fallback signatures, embedded fallback signer key material, parser paths that can mis-handle structured JSON, and unbounded replay/state growth in hot paths.

## Scope
In scope:
- Remove production hardcoded fallback signer private key constants from library code paths.
- Remove silent fallback from signer failures to deterministic/non-cryptographic signatures.
- Correct signature algorithm labeling and deterministic profile taxonomy constants.
- Replace targeted hand-rolled JSON field extraction in runtime-facing MCP/SDK request parsing with `serde_json` parsing in touched endpoints.
- Add deterministic capacity controls to touched replay/state growth structures.

Out of scope:
- Full protocol redesign.
- Full kamn-core crate decomposition.
- Async redesign of all SDK APIs.
- Kubernetes production manifest expansion.

## Acceptance Criteria
### AC-1 No hardcoded fallback private key in production source paths
Given kamn-core production source,
When signer key resolution is invoked,
Then no hardcoded fallback private key constant is compiled in production paths.

### AC-2 No silent deterministic fallback when cryptographic signer fails
Given signer resolution/signing failure,
When transaction signing is requested,
Then call sites return explicit error and do not emit deterministic fallback signatures.

### AC-3 Signature algorithm taxonomy is internally consistent
Given signature profile constants,
When signature metadata is emitted,
Then algorithm labels no longer claim `ed25519` for secp256k1 or deterministic non-cryptographic profiles.

### AC-4 JSON parsing in touched runtime request paths is structured and escape-safe
Given JSON payloads containing escaped quotes/embedded marker strings/whitespace variants,
When parsing request fields,
Then extracted values remain correct and deterministic via `serde_json` parsing.

### AC-5 Replay/state growth controls are bounded for touched structures
Given replay/state guard maps in touched transaction/auth paths,
When sustained traffic is applied,
Then in-memory tracking does not grow without bound.

## Conformance Cases
- C-01 (Security, AC-1): source grep/test confirms fallback key constants removed from kamn-core production sources.
- C-02 (Functional, AC-2): signer failure path tests assert explicit error (no deterministic signature fallback).
- C-03 (Functional, AC-3): signature profile metadata tests assert corrected algorithm/taxonomy labels.
- C-04 (Conformance, AC-4): JSON parser tests cover escaped quote, marker-in-string, and whitespace variants.
- C-05 (Performance/Safety, AC-5): replay/state guard tests verify deterministic capacity bound and eviction behavior.

## Success Metrics / Observable Signals
- No production signer fallback key constants in kamn-core source.
- Transaction/signature tests fail closed on signer failures.
- JSON parser conformance tests pass with adversarial escaped-input fixtures.
- Bounded replay/state tracking tests pass under over-capacity inserts.
