#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
README_FILE="$ROOT_DIR/README.md"

if [ ! -f "$README_FILE" ]; then
  echo "README contract failed: README.md is missing at repository root." >&2
  exit 1
fi

required_headers=(
  "# KAMN"
  "## What This Repository Contains"
  "## Quickstart"
  "## Workflow"
  "## Key Links"
)

for header in "${required_headers[@]}"; do
  if ! grep -Fq "$header" "$README_FILE"; then
    echo "README contract failed: missing header '$header'." >&2
    exit 1
  fi
done

required_snippets=(
  "cargo fmt --check"
  "cargo clippy -- -D warnings"
  "cargo test"
  "make smoke-live-network"
  "live-network-smoke-report.json"
  "make deep-live-network"
  "live-network-pilot-report.json"
  "make demo-localhost-transport"
  "run_localhost_signed_demo.sh"
  "run_localhost_signed_demo.sh --help"
  "--timeout-seconds"
  "run_localhost_signed_integration_harness.sh --scenario signature-mismatch"
  "run_localhost_signed_integration_contract_lane.sh"
  "kamn.sdk.localhost-signed.integration-contract.v1"
  "run_localhost_signed_integration_contract_lane_tests"
  "run_localhost_bridge_demo_evidence_contract_lane.sh"
  "run_localhost_bridge_demo_evidence_deep_lane.sh"
  "kamn.bridge.localhost-demo-evidence.v1"
  "Node.js 22"
  "bash scripts/frontend/test_dashboard_package_runtime_compat.sh"
  "KAMN_DASHBOARD_NODE_BIN"
  "KAMN_DASHBOARD_FALLBACK_NODE_CMD"
  "AGENTS.md"
  "PRD.md"
)

for snippet in "${required_snippets[@]}"; do
  if ! grep -Fq -- "$snippet" "$README_FILE"; then
    echo "README contract failed: missing snippet '$snippet'." >&2
    exit 1
  fi
done

echo "README contract tests passed."
