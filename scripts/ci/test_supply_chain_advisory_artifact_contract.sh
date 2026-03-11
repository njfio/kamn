#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORKFLOW_FILE="$ROOT_DIR/.github/workflows/ci-supply-chain-advisory.yml"
CI_TOOLS_FILE="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
SCOPE_POLICY_FILE="$ROOT_DIR/scripts/ci/test_workflow_scope_policy.sh"

for required in \
  'name: Ensure advisory filesystem report exists' \
  'name: Ensure advisory image report exists' \
  'name: Ensure advisory SBOM report exists' \
  'placeholder_due_to_missing_output' \
  'generated_by_scan' \
  '"report": "trivy-fs"' \
  '"report": "trivy-image"' \
  '"value": "sbom"'; do
  if ! grep -Fq "$required" "$WORKFLOW_FILE"; then
    echo "expected supply-chain advisory artifact contract marker in workflow: $required" >&2
    exit 1
  fi
done

if grep -Fq 'name: Verify advisory reports exist' "$WORKFLOW_FILE"; then
  echo 'legacy advisory report verification step must be removed in favor of explicit placeholder generation' >&2
  exit 1
fi

if ! grep -Fq 'bash "$ROOT_DIR/scripts/ci/test_supply_chain_advisory_artifact_contract.sh"' "$CI_TOOLS_FILE"; then
  echo 'expected ci tools fast-mode entrypoint to run supply-chain advisory artifact contract test' >&2
  exit 1
fi

if ! grep -Fq 'test_supply_chain_advisory_artifact_contract.sh' "$SCOPE_POLICY_FILE"; then
  echo 'expected workflow scope policy coverage to reference supply-chain advisory artifact contract test' >&2
  exit 1
fi
