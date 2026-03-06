# DID Format Standardization

## Intent
Inventory the current DID format divergence in the repository and record the target canonical
format before any parser or public-contract changes are attempted.

## Standard Markers
- `did_format_current_canonical=kamn:did:{role}:{id}`
- `did_format_divergent_shape=did:kamn:{role}:{id}`
- `did_format_target_standard=kamn:did:{role}:{id}`
- `did_format_divergent_consumer_count=0`
- `did_format_followup_scope=parser-compatibility-decision-only`
- `did_format_divergent_reference_scope=inventory-only`
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

Active divergent consumers are now reduced to zero outside this document's intentional
divergence-description section.

This document still names `did:kamn:...` because it records the deprecated shape for migration and
public-contract planning, not because the repository keeps active example call sites on that shape.

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

- decide whether any legacy `did:kamn:...` inputs need a temporary compatibility window or direct
  fail-closed rejection under a separate approved issue
- if parser or API-contract enforcement changes are approved, update migration guidance and public
  contract documentation in the same issue

## Decision Gate

Any implementation issue that changes accepted DID wire formats, parser behavior, or externally
visible runtime values must be treated as a public-contract change and approved before code lands.
