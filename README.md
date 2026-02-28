# KAMN

KAMN (Kolme AI Agent Messaging Network) is a privacy-first, auditable coordination layer for autonomous agents.

This repository contains the Rust runtime/core crates, live-node scaffolding, SDK surfaces, and CI/governance tooling used to evolve the protocol safely.

## What This Repository Contains

- `crates/kamn-core`: protocol/domain logic and contract suites
- `crates/kamn-node`: node runtime entrypoint and service API
- `crates/kamn-sdk`: Rust SDK client surface
- `crates/kamn-agent-lib`: agent-facing auth/identity helpers
- `crates/kamn-kolme`: Kolme live provider integration layer
- `scripts/`: CI, contract lanes, and deterministic validation utilities
- `docs/`: architecture, operations, security, and planning references

## Quickstart

Prerequisites:
- Rust toolchain (`cargo`, `rustc`)
- Bash
- Python 3
- Node.js/npm (for dashboard/TS-related lanes)

Core validation:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
cargo test -p kamn-node
bash scripts/ci/test_ci_tools.sh
```

Live HTTPS dependency posture checks (`kamn-core`):
- Dependencies:
- `rustls`
- `rustls-pemfile`
- `webpki-roots`

```bash
cargo check -p kamn-core --features live-https
cargo check -p kamn-core --no-default-features
```

Fast repository lanes:

```bash
make check
make test
make ci-tools
```

Live-network smoke entrypoints:

```bash
make smoke-live-network
make deep-live-network
make demo-localhost-transport
```

## Workflow

1. Open/confirm issue + milestone scope.
2. Author `specs/<issue-id>/{spec,plan,tasks}.md`.
3. Run RED -> GREEN -> REFACTOR for each task.
4. Update docs/specs in the same PR when behavior changes.
5. Keep `cargo fmt`, `clippy`, and scoped tests green before push.

Repository process contract:
- `AGENTS.md`

## Architecture Map

Start here for system navigation:
- `docs/architecture/README.md`

Core architecture references:
- `docs/architecture/runtime-layout.md`
- `docs/architecture/service-runtime.md`
- `docs/architecture/kamn-core-module-map.md`
- `docs/architecture/kamn-node-module-map.md`
- `docs/foundation/kolme-runtime-architecture.md`
- `docs/foundation/runtime-network.md`
- `docs/architecture/adr-kamn-core-live-tls-transport.md`

## Contract Reference

Detailed command matrices, contract markers, policy snippets, and lane-specific references are maintained in:

- `docs/developer/readme-contract-reference.md`

This keeps the root README onboarding-focused while preserving deterministic contract markers in a stable docs location.

## Key Links

- CI strategy: `docs/ci/strategy.md`
- Engineering hardening wave: `docs/planning/engineering-hardening-wave.md`
- Runtime/live ops: `docs/planning/live-network-wave.md`
- Kolme devnet ops: `docs/planning/kolme-devnet-ops.md`
- Rustdoc publishing guide: `docs/developer/rustdoc-publishing.md`
- Missing-docs policy checker: `scripts/ci/check_kamn_core_missing_docs_policy.sh`
- Security guidance: `docs/security/secure-coding.md`
- TLS hardening: `docs/security/tls-hardening.md`
