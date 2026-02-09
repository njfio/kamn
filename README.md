# KAMN

KAMN (Kolme AI Agent Messaging Network) is a privacy-first, auditable coordination layer for autonomous agents. This repository contains the Rust core state machine, node/runtime scaffolding, SDK surfaces, deterministic fixture lanes, and CI policy tooling used to evolve the protocol safely.

## What This Repository Contains

- `crates/kamn-core`: core protocol/domain logic and contract tests.
- `crates/kamn-node`: node/runtime entrypoint scaffolding.
- `crates/kamn-sdk`: Rust SDK surface.
- `scripts/`: deterministic validation lanes and CI helper tooling.
- `fixtures/`: replay/contract fixtures used by fast and deep lanes.
- `docs/foundation/`: implementation contracts mapped to PRD scope.

## Quickstart

### Prerequisites

- Rust toolchain (`cargo`, `rustc`)
- Bash shell
- Node.js/npm (only needed for dashboard/TypeScript lanes)

### Validate Local Environment

```bash
# Format
cargo fmt --check

# Lint (strict)
cargo clippy -- -D warnings

# Core tests
cargo test

# CI tool regression suite (fast/deep routing guards, script contracts)
bash scripts/ci/test_ci_tools.sh
```

### Fast Make Lanes

```bash
# Fast static gates
make check

# Default bounded test lane
make test

# Two-process localhost signed-message demo
make demo
```

Deep/scheduled lanes remain opt-in via scripts in `scripts/sdk/` and `scripts/ci/`.

### Run A Focused Core Slice

```bash
cargo test -p kamn-core --test trust_score_engine --test trust_score_engine_docs --test reputation_state_model_docs
bash scripts/ci/test_select_targets.sh
```

### Run A Local End-to-End Demo

```bash
bash scripts/sdk/run_local_e2e_demo.sh
```

### Run Triadic Devnet Smoke (Kolme)

```bash
bash scripts/kolme/run_triadic_devnet_smoke.sh --output-file /tmp/triadic-devnet-markers.txt
python3 scripts/kolme/validate_triadic_devnet_smoke.py --fixture fixtures/kolme_compatibility/devnet_smoke_markers.json --marker-file /tmp/triadic-devnet-markers.txt --output-json /tmp/triadic-devnet-report.json
```

## Workflow

All code changes are issue-first and follow strict Red → Green → Refactor → Regression TDD. Before implementation:

1. Create or select a GitHub task issue with required labels.
2. Move the issue to `status:in-progress`.
3. Create a branch: `codex/issue-<id>-<short-slug>`.
4. Log progress comments on the issue using the required status template.

Canonical contributor rules are in `.github/CONTRIBUTING.md` (`AGENTS.md` remains a compatibility redirect).

## Key Links

- `.github/CONTRIBUTING.md`: mandatory execution contract (issue hierarchy, TDD, PR standards).
- `AGENTS.md`: compatibility redirect for agent tooling.
- `PRD.md`: product requirements and phase scope baseline.
- `docs/foundation/`: domain contracts used by docs tests and release gates.
- `.github/workflows/`: CI lane orchestration.
