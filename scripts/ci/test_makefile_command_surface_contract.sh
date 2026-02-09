#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MAKEFILE="$ROOT_DIR/Makefile"

if [ ! -f "$MAKEFILE" ]; then
  echo "Makefile command-surface contract failed: Makefile is missing." >&2
  exit 1
fi

required_targets=(
  "check"
  "test"
  "smoke-live-network"
  "deep-live-network"
  "demo"
  "demo-localhost-transport"
  "ci-tools"
)

for target in "${required_targets[@]}"; do
  if ! grep -q "^${target}:" "$MAKEFILE"; then
    echo "Makefile command-surface contract failed: missing target '${target}'." >&2
    exit 1
  fi
done

required_help_snippets=(
  "make check"
  "make test"
  "make smoke-live-network"
  "make deep-live-network"
  "make demo"
  "make demo-localhost-transport"
  "make ci-tools"
)

for snippet in "${required_help_snippets[@]}"; do
  if ! grep -Fq -- "$snippet" "$MAKEFILE"; then
    echo "Makefile command-surface contract failed: help output missing snippet '${snippet}'." >&2
    exit 1
  fi
done

required_command_snippets=(
  "cargo fmt --check"
  "cargo clippy --workspace --all-targets --all-features -- -D warnings"
  "cargo test"
  "bash scripts/runtime/run_live_network_smoke_lane.sh --output-json /tmp/live-network-smoke-report.json"
  "bash scripts/runtime/run_live_network_pilot_deep_lane.sh --event-name workflow_dispatch --output-json /tmp/live-network-pilot-report.json"
  "bash scripts/sdk/run_localhost_signed_demo.sh"
  "bash scripts/ci/test_ci_tools.sh"
)

for snippet in "${required_command_snippets[@]}"; do
  if ! grep -Fq -- "$snippet" "$MAKEFILE"; then
    echo "Makefile command-surface contract failed: missing command snippet '${snippet}'." >&2
    exit 1
  fi
done

echo "Makefile command-surface contract tests passed."
