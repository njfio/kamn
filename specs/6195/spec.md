# Spec: Issue 6195 - Add Baseline Unit Coverage for Data Layer M0-M5

- Issue: #6195
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P1
- Area: qa

## Problem Statement

Foundational data-layer modules M0 through M5 currently have no in-module unit tests,
leaving append-only, merkle, ABAC, search, escrow, and vector paths under-verified.

## Scope

In scope:
1. Add baseline unit tests in each module:
   - `data_layer_m0.rs`
   - `data_layer_m1.rs`
   - `data_layer_m2_gateway_access.rs`
   - `data_layer_m3_blind_index_search.rs`
   - `data_layer_m4_escrow_integration.rs`
   - `data_layer_m5_vector_integration.rs`
2. Cover one happy-path behavior and one fail-closed behavior in each module.

Out of scope:
1. Full property/fuzz suite for all data-layer modules.
2. Service-API wiring for M0-M5.

## Acceptance Criteria

### AC-1 M0-M5 Have Concrete Unit Tests
Given the six M0-M5 source modules,
When tests are collected,
Then each module contains at least one `#[test]` validating behavior.

### AC-2 Core Failure Paths Are Covered
Given representative invalid inputs per module,
When tested,
Then APIs fail closed with deterministic error variants.

### AC-3 Scoped Core Test Lanes Pass
Given updated modules,
When running scoped `kamn-core` test lanes,
Then all added tests pass without regressions.

## Conformance Cases

- C-01 (AC-1, Unit): `data_layer_m0::tests::unit_data_layer_m0_append_ledger_verifies_hash_chain`
- C-02 (AC-1, Unit): `data_layer_m1::tests::unit_data_layer_m1_merkle_batch_proof_verifies`
- C-03 (AC-1, Unit): `data_layer_m2_gateway_access::tests::unit_data_layer_m2_session_authenticate_succeeds_for_valid_request`
- C-04 (AC-1, Unit): `data_layer_m3_blind_index_search::tests::unit_data_layer_m3_register_and_search_blind_index_exact_match`
- C-05 (AC-1, Unit): `data_layer_m4_escrow_integration::tests::unit_data_layer_m4_escrow_transition_flow_reaches_disputed`
- C-06 (AC-1, Unit): `data_layer_m5_vector_integration::tests::unit_data_layer_m5_append_and_semantic_query_rank_results`
