# Plan: Issue #4735

Status: Reviewed
Issue: #4735

## Approach

1. Restore deterministic compilation boundaries by ensuring `block_pipeline_support` owns required
   helper implementations/imports after root extraction cleanup.
2. Keep root `block_pipeline.rs` as orchestration/re-export surface with explicit submodule wiring.
3. Harden SDK TCP sender shutdown handling by accepting benign close-race error kinds after
   successful write/flush.
4. Replace service API lifecycle limiter projection `expect()` sites with deterministic fallback
   policy structs.
5. Wire compose service API TLS env and HTTPS healthchecks, then update docs and deployment asset
   contract checks to fail closed on drift.

## Affected Modules

- `crates/kamn-core/src/block_pipeline.rs`
- `crates/kamn-core/src/block_pipeline/block_pipeline_support.rs`
- `crates/kamn-sdk/src/tcp.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `deploy/docker-compose.yml`
- `deploy/certs/README.md`
- `docs/deployment/docker.md`
- `docs/ops/deployment.md`
- `scripts/deploy/test_deployment_assets.sh`

## Risks / Mitigations

- Risk: extraction follow-through can leave dead or duplicated type stacks.
  - Mitigation: compile `kamn-core`, run extraction boundary contracts, and run transport-fed and
    gossip ingest integration selectors.

- Risk: SDK shutdown handling could mask non-benign transport failures.
  - Mitigation: only suppress explicit benign `std::io::ErrorKind` variants tied to peer-close
    races and keep other failures unchanged.

- Risk: compose TLS changes can drift from deployment validation scripts/docs.
  - Mitigation: update scripts/docs in the same change and run deployment asset + live validation.

## Interface Contract

- No external API signature changes for exported Rust interfaces.
- No wire format changes.
- Deployment contract changes are additive governance requirements:
  - service API TLS env markers required in compose.
  - HTTPS healthcheck endpoints required for role services.

## ADR

Not required (no new dependency, no protocol or architecture decision change).
