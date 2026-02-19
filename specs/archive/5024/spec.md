# Issue #5024 Spec

- Title: Task: M8 deliver crypto-shredding, retention policy enforcement, and legal-hold controls
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD M8 requires deterministic contracts for crypto-shredding, retention-policy
evaluation, and legal-hold gating. The existing codebase has generic retention
helpers but no dedicated M8 compliance surface that models message-level CEK
destruction, legal-hold precedence, and owner-scoped retention worker outputs.

PRD mapping:
- Section 11.1 (crypto-shredding semantics and irreversibility)
- Section 11.2 (retention classes, hourly worker behavior, legal hold precedence)
- Section 11.3 (right-to-erasure fail-closed via CEK destruction)
- Section 11.4 (retention window extensions for escrow/dispute lifecycle)
- Milestone table M8 deliverables (retention worker + crypto-shred controls)

## Acceptance Criteria
- AC-1: Crypto-shredding contract destroys wrapped CEKs, sets `shredded_at`, and
  preserves append-only integrity markers (`content_hash`, `hash_chain_prev`).
- AC-2: Retention-policy contract evaluates class windows deterministically
  (ephemeral/standard/extended/permanent/legal-hold) with optional lifecycle
  extension seconds for escrow/dispute windows.
- AC-3: Legal-hold controls block shredding and retention-worker shredding
  candidates until legal hold is explicitly released.
- AC-4: Cross-owner retention/shredding operations are denied fail-closed with
  stable reason markers.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust M8 compliance module for message-level retention + legal-hold
  controls and crypto-shredding transitions.
- Conformance tests for CEK tombstoning, retention due evaluation, legal-hold
  precedence, and owner-scope fail-closed guards.
- Public API exports for downstream M9+ pipeline integration.

Out of scope:
- Live PostgreSQL worker scheduling and SQL DDL migrations.
- New shell/python/workflow/template orchestration.
- New dependencies or wire/protocol format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Register one message and execute crypto-shred | Wrapped keys replaced with tombstone marker, `shredded_at` set, integrity fields unchanged |
| C-02 | AC-2 | Conformance | Evaluate retention due set across classes/timestamps/extensions | Deterministic ordering and eligibility outputs by class window rules |
| C-03 | AC-3 | Conformance | Apply legal hold, attempt shred + retention evaluation, release hold, retry | Hold blocks shred/due candidates until release; release re-enables policy flow |
| C-04 | AC-4 | Regression | Perform retention/shred operations with mismatched requester owner | Fail-closed owner-scope violation with stable reason marker |
| C-05 | AC-4 | Regression | Re-shred previously shredded message | Deterministic `AlreadyShredded` error |
| C-06 | AC-5 | Regression | Inspect issue diff paths | No shell/python/workflow/template path changes |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m8_compliance_lifecycle`
- `cargo test -p kamn-core spec_c0`
- `cargo test -p kamn-core`
- Shell governance scripts are not required because shell/workflow surfaces are unchanged.

## Success Metrics
- M8 contract suite exposes deterministic retention/legal-hold/shred APIs via `kamn_core`.
- All ACs map to passing `spec_c0x_*` tests.
- Shell-to-Rust ratio improves or remains neutral from Rust-only changes.

## Verification
| AC | Result | Tests/Evidence |
|---|---|---|
| AC-1 | ✅ | `spec_c01_crypto_shred_replaces_wrapped_keys_and_preserves_integrity_markers` |
| AC-2 | ✅ | `spec_c02_retention_due_windows_are_deterministic_across_classes_and_extensions` |
| AC-3 | ✅ | `spec_c03_legal_hold_blocks_shredding_and_due_candidates_until_released` |
| AC-4 | ✅ | `spec_c04_cross_owner_operations_are_denied_fail_closed`, `spec_c05_double_shred_is_rejected_with_stable_error` |
| AC-5 | ✅ | `git diff --name-only` confirms no shell/python/workflow/template path changes |

Executed commands:
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `cargo test -p kamn-core --test data_layer_m8_compliance_lifecycle`
- `cargo test -p kamn-core`
