# R27.30 Milestone Index

- Milestone: `R27.30 Async API runtime, networked peer transport, and durable block-pipeline governance`
- GitHub milestone: `#64`
- Parent epic: `#4310`

## Scope
Close monolithic ownership and deterministic governance gaps across runtime networking and block-pipeline durability contracts.

## Active Task Track
- `#4693` Task: decompose `p2p_transport.rs` and `block_pipeline.rs` into focused modules while preserving deterministic fail-closed behavior.

## Verification Baseline
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- targeted `cargo test -p kamn-core ...` selectors mapped by issue-level conformance cases

## Evidence and Docs
- `docs/foundation/runtime-network.md`
