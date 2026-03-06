# DID Format Standardization

## Intent
Inventory the current DID format divergence in the repository and record the target canonical
format before any parser or public-contract changes are attempted.

## Standard Markers
- `did_format_current_canonical=kamn:did:{role}:{id}`
- `did_format_divergent_shape=did:kamn:{role}:{id}`
- `did_format_target_standard=kamn:did:{role}:{id}`
- `did_format_followup_scope=standardize-runtime-and-doc-consumers`
- `did_format_public_contract_gate=approval-required`

## Current State

### Canonical Shape
The canonical shape already enforced by `kamn-core` and `kamn-types` is:

- `kamn:did:{role}:{id}`
- agent example: `kamn:did:agent:alpha`
- operator example: `kamn:did:operator:node-1`

Concrete canonical consumers include:

- `crates/kamn-core/src/did.rs`
- `crates/kamn-types/src/lib.rs`
- `docs/sdk/rust-sdk.md`

### Divergent Shape
The repository still contains a divergent shape:

- `did:kamn:{role}:{id}`
- agent example: `did:kamn:agent:alpha`

Concrete divergent consumers currently include:

- `did_format_divergent_consumer=crates/kamn-node/src/main_tests/runtime_tests/runtime_mode_and_transport_profile_tests.rs`
- `did_format_divergent_consumer=crates/kamn-node/src/main_tests/cli_contract_tests.rs`
- `did_format_divergent_consumer=docs/foundation/node-runtime-cli.md`

These usages show the divergence is not limited to docs; it still appears in node CLI/test
surfaces even after the Kolme/runtime-request helper fixtures were standardized.

## Target Standard
Future implementation work should standardize on:

- `kamn:did:{role}:{id}`

Reasoning:

- this is the format parsed and validated by `kamn-core::did::{AgentDid, KamnDid}`
- this is the format exposed by `kamn-types` as the shared identity boundary
- this keeps crate, SDK, and architecture docs aligned with the existing canonical parser surface

## Non-Goals For Issue 6489

- no parser or runtime behavior changes
- no automatic migration of existing `did:kamn:...` call sites
- no compatibility shims, redirects, or dual-write logic

## Follow-Up Direction

- decide whether divergent `did:kamn:...` inputs need a temporary compatibility window or direct
  cleanup
- update remaining node CLI/proposal/test call sites to emit canonical `kamn:did:{role}:{id}`
  values
- only then consider parser or API-contract enforcement changes under a separate approved issue

## Decision Gate

Any implementation issue that changes accepted DID wire formats, parser behavior, or externally
visible runtime values must be treated as a public-contract change and approved before code lands.
