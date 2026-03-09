#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORKFLOW="$ROOT_DIR/.github/workflows/ci-supply-chain-advisory.yml"

if [ ! -f "$WORKFLOW" ]; then
  echo "expected advisory supply-chain workflow file at .github/workflows/ci-supply-chain-advisory.yml" >&2
  exit 1
fi

require_workflow_marker() {
  local marker="$1"
  local message="$2"
  if ! grep -Fq "$marker" "$WORKFLOW"; then
    echo "$message" >&2
    exit 1
  fi
}

require_workflow_marker "name: Supply-Chain Advisory" "expected advisory workflow name marker"
require_workflow_marker "workflow_dispatch:" "expected workflow_dispatch trigger for advisory workflow"
require_workflow_marker "pull_request:" "expected pull_request trigger for advisory workflow"
require_workflow_marker "schedule:" "expected scheduled trigger for advisory workflow"
require_workflow_marker "continue-on-error: true" "expected advisory steps to remain non-blocking"
require_workflow_marker "aquasecurity/trivy-action" "expected Trivy action usage for advisory scanning"
require_workflow_marker "scanners: vuln,secret,license" "expected filesystem advisory scan to include vuln, secret, and license scanners"
require_workflow_marker "docker build -t kamn-supply-chain-advisory:${{ github.sha }} ." "expected advisory workflow to build the repo Docker image"
require_workflow_marker "format: cyclonedx" "expected advisory workflow to generate a CycloneDX SBOM artifact"
require_workflow_marker "scripts/ci/check_workspace_license_policy.py" "expected advisory workflow to reuse the workspace license policy checker"
require_workflow_marker "ci-supply-chain-advisory-trivy-fs.json" "expected advisory workflow to upload filesystem advisory artifact"
require_workflow_marker "ci-supply-chain-advisory-trivy-image.json" "expected advisory workflow to upload image advisory artifact"
require_workflow_marker "ci-supply-chain-advisory-sbom.cdx.json" "expected advisory workflow to upload SBOM artifact"
require_workflow_marker "ci-supply-chain-advisory-license.json" "expected advisory workflow to upload license advisory artifact"

echo "supply-chain advisory workflow contract tests passed."
