# DID Format Standardization

## Intent
Inventory the current DID format divergence in the repository and record the target canonical
format before any parser or public-contract changes are attempted.

## Standard Markers
- `did_format_current_canonical=kamn:did:{role}:{id}`
- `did_format_divergent_shape=did:kamn:{role}:{id}`
- `did_format_target_standard=kamn:did:{role}:{id}`
- `did_format_divergent_consumer_count=0`
- `did_format_followup_scope=approved-enforcement-issue-only`
- `did_format_divergent_reference_scope=inventory-only`
- `did_format_legacy_input_policy=direct-fail-closed`
- `did_format_policy_boundary=parser-and-api-ingress`
- `did_format_runtime_output_policy=canonical-only`
- `did_format_normalization_policy=no-silent-rewrite`
- `did_format_policy_implementation_gate=approved-followup-required`
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

## Legacy Input Policy

- `did_format_legacy_input_policy=direct-fail-closed`
- `did_format_policy_boundary=parser-and-api-ingress`
- `did_format_runtime_output_policy=canonical-only`
- `did_format_normalization_policy=no-silent-rewrite`
- `did_format_policy_implementation_gate=approved-followup-required`

Policy decision:

- legacy `did:kamn:...` inputs are not approved as a long-term compatibility shape
- the intended enforcement target is direct fail-closed rejection at shared parser boundaries and
  API ingress points that accept DID strings
- no shared parser or ingress path should silently rewrite `did:kamn:...` into canonical
  `kamn:did:...` behind the caller's back
- runtime, API, CLI, SDK, and documentation outputs must remain canonical-only:
  `kamn:did:{role}:{id}`
- any implementation issue that enforces this policy must first audit affected parser helpers,
  ingress surfaces, rollout notes, and migration guidance under a separate approved public-contract
  change

## Non-Goals For Issue 6489

- no parser or runtime behavior changes
- no automatic migration of existing `did:kamn:...` call sites
- no compatibility shims, redirects, or dual-write logic

## Follow-Up Direction

- audit the exact parser helpers, API ingress surfaces, and runtime boundaries that still need an
  explicit enforcement plan
- implement direct fail-closed rejection only under a separate approved public-contract issue that
  updates migration guidance and rollout communication in the same change

## Decision Gate

Any implementation issue that changes accepted DID wire formats, parser behavior, or externally
visible runtime values must be treated as a public-contract change and approved before code lands.
