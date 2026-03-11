#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORKFLOW_FILE="$ROOT_DIR/.github/workflows/ci-supply-chain-advisory.yml"
CI_TOOLS_FILE="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
SCOPE_POLICY_FILE="$ROOT_DIR/scripts/ci/test_workflow_scope_policy.sh"
HELPER_FILE="$ROOT_DIR/scripts/ci/ensure_advisory_report.py"

for required in \
  'name: Ensure advisory filesystem report exists' \
  'name: Ensure advisory image report exists' \
  'name: Ensure advisory SBOM report exists' \
  'trivy-fs' \
  'trivy-image' \
  'sbom'; do
  if ! grep -Fq "$required" "$WORKFLOW_FILE"; then
    echo "expected supply-chain advisory artifact contract marker in workflow: $required" >&2
    exit 1
  fi
done

if ! grep -Fq 'python3 scripts/ci/ensure_advisory_report.py' "$WORKFLOW_FILE"; then
  echo 'expected advisory workflow to route placeholder generation through ensure_advisory_report.py' >&2
  exit 1
fi

for helper_marker in \
  'placeholder_due_to_missing_output' \
  'generated_by_scan' \
  '"bomFormat": "CycloneDX"' \
  '"report": payload_kind' \
  ; do
  if ! grep -Fq "$helper_marker" "$HELPER_FILE"; then
    echo "expected advisory artifact helper marker in ensure_advisory_report.py: $helper_marker" >&2
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
