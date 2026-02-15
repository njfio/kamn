# KAMN: Production Service Next Steps (Truth-Refresh)

**Document date:** 2026-02-14 (refreshed 2026-02-15)
**Authoritative tracker:** #3333
**Scope:** reconcile roadmap claims with current implementation and open issue chains.

## Why This Refresh Exists
The original 2026-02-14 draft mixed a historical baseline snapshot with forward planning.
That produced contradictory signals (for example, marking ingress/storage as scaffolding when those chains were already delivered in later waves).

This refreshed version separates:
- delivered roadmap items and their closed chains,
- remaining open production gaps and their active issue chains,
- explicit local-heavy validation scope and CI cost boundaries.

## Status Truth Snapshot

| Item | Current Truth | Primary Chain |
|---|---|---|
| 1. HTTP ingress runtime | Delivered (`axum` server + auth/ws integration) | #3254 -> #3305 -> #3306 |
| 2. Persistent storage | Delivered (sqlite backend adapters + migration parity) | #3221 -> #3309 -> #3310 |
| 3. Real P2P transport | Open (libp2p transport hardening still active) | #3228 -> #3229 -> #3313 |
| 4. Transport-fed consensus pipeline | Open follow-on (real gossip-fed convergence still active) | #3228 -> #3413 -> #3414 |
| 5. Serde API/domain serialization | Delivered | #3263 -> #3264 -> #3265 |
| 6. Prometheus/metrics contract | Delivered | #3263 -> #3268 -> #3269 |
| 7. Graceful shutdown path | Delivered | #3263 -> #3272 -> #3273 |
| 8. Request validation + route error contracts | Delivered | #3263 -> #3276 -> #3277 |
| 9. Compose topology hardening | Delivered | #3280 -> #3285 -> #3286 |
| 10. Combined runtime mode (`full`) | Delivered | #3280 -> #3281 -> #3282 |
| 11. Real I/O integration test matrix | Delivered | #3289 -> #3290 -> #3291 |
| 12. Structured API error envelope | Delivered | #3289 -> #3294 -> #3295 |
| 13. Rate limiting and request caps | Delivered | #3289 -> #3298 -> #3299 |

## Delivered Chains (Closed or Downstream-Closed)

### Item 1: Axum ingress
- Delivered chain: `#3254 -> #3305 -> #3306 -> (#3307, #3308)`.
- Runtime now serves through `axum::serve(...)` with route wiring in `crates/kamn-node/src/service_api_endpoint.rs`.

### Item 2: Sqlite persistence
- Delivered chain: `#3221 -> #3309 -> #3310 -> (#3354, #3355, #3311, #3312)`.
- Sqlite backend and adapters are live in `kamn-core` (`SqliteStoreBackend`, `Sqlite*SnapshotStore`).

### Items 5-13 (infrastructure maturity tranche)
- Serde: `#3263 -> #3264 -> #3265 -> (#3266, #3267)`.
- Prometheus metrics: `#3263 -> #3268 -> #3269 -> (#3270, #3271)`.
- Graceful shutdown: `#3263 -> #3272 -> #3273 -> (#3274, #3275)`.
- Request validation: `#3263 -> #3276 -> #3277 -> (#3278, #3279)`.
- Compose topology: `#3280 -> #3285 -> #3286 -> (#3360, #3361, #3287, #3288)`.
- Runtime-mode full: `#3280 -> #3281 -> #3282 -> (#3358, #3359, #3283, #3284)`.
- Real-I/O integration matrix: `#3289 -> #3290 -> #3291 -> (#3362, #3363, #3292, #3293)`.
- Structured errors: `#3289 -> #3294 -> #3295 -> (#3364, #3365, #3296, #3297)`.
- Rate limiting: `#3289 -> #3298 -> #3299 -> (#3366, #3367, #3300, #3301)`.

## Remaining Open Gaps (Active)

### Item 3: Real libp2p transport delivery
- Open chain: `#3228 -> #3229 -> #3313 -> (#3356, #3314, #3315, #3319, #3470)`.
- Current focus:
  - deterministic swarm behavior composition,
  - lifecycle/discovery fail-closed evidence,
  - local-heavy live lane validation with CI fast-gate exclusion,
  - Task #3470 live transport fault matrix policy lane:
    `scripts/runtime/validate_live_transport_fault_matrix_live_contract_lane.sh`,
    `scripts/runtime/check_live_transport_fault_matrix_live_policy.sh`.

### Item 4 follow-on: Transport-fed consensus convergence
- Open chain: `#3228 -> #3413 -> #3414 -> (#3443, #3444, #3446, #3447, #3448)`.
- Current focus:
  - block pipeline consumption from real transport adapters,
  - canonical fork-choice/reconciliation under partition-rejoin/publish-drop/churn convergence drills,
  - deterministic policy/taxonomy reason-code contracts,
  - release go/no-go artifact gating on transport-fed convergence + fault-matrix evidence.
- Status progression note:
  - delivered in this chain: `#3443`, `#3444`, `#3446`, `#3447`
  - remaining in this chain: `#3448`

### Composed full-stack E2E against `kolme_fork`
- Open chain: `#3333 -> #3419 -> #3420 -> (#3432, #3433, #3434)`.
- Scope:
  - composed local-heavy integration evidence bundle,
  - release go/no-go linkage,
  - architecture/documentation lineage.
- Remaining work is tracked under the parent story until stacked PR chain merges.

### Runtime observability reason-code/checkpoint projection
- Active chain: `#3333 -> #3471 -> #3472 -> #3473 -> #3474 -> #3490`.
- Scope:
  - include deterministic `reason_code` and checkpoint-failure counters across daemon and `kolme-live` runtime reports,
  - project the same fields into `/metrics`, `/healthz`, and `/metrics.stream`,
  - keep report text/json output deterministic while expanding observability payload contracts,
  - maintain local observability scrape failure-drill contract-lane parity (`scripts/runtime/validate_local_observability_scrape_live_contract_lane.sh`, `scripts/runtime/test_validate_local_observability_scrape_live.sh`, `scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh`).
- Validation:
  - local-focused targeted tests in `kamn-node` for daemon and `kolme-live` runtime execution status mapping,
  - endpoint rendering contract tests for metrics/health/stream payload parity,
  - readiness degradation/failure-drill marker coverage in the local observability scrape lane and policy checker.

### Docs truth synchronization (this tranche)
- Open chain: `#3333 -> #3424 -> #3425 -> #3426`.
- Scope:
  - keep this document and CI docs-contract guards synchronized with real state.

## Cost and CI Policy Boundaries
- Heavy local integration run-mode lanes remain excluded from `ci-fast-gate` and fast `ci-tools` blocks.
- Deterministic dry-run contract checks remain in PR path.
- Local-heavy/live-node validations remain opt-in and bounded for cost control.
- Reference policy doc: `docs/ci/strategy.md`.

## Validation Commands (Low-Cost Truth Guard)
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `bash scripts/ci/test_kolme_live_integration_architecture_contract.sh`
- `bash scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh`

## Historical Baseline (Superseded)
The earlier 2026-02-14 draft language that described ingress/storage/runtime composition as unresolved scaffolding is retained only as historical context and is superseded by the issue-chain status above.

Use #3333 as the single source of truth for roadmap status transitions.
