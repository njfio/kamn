#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/deploy/run_dr_evidence_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/deploy/run_dr_evidence_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/deploy/dr_evidence_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/deploy_dr_evidence_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected DR evidence fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected DR evidence deep-lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected DR evidence shared contract-lane module to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "dr evidence contract lane tests passed." "$TMP_OUT"; then
  echo "expected DR evidence contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected DR evidence contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected DR evidence contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected DR evidence wrapper to resolve deploy manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "dr_evidence_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected DR evidence manifest to dispatch shared contract module" >&2
  exit 1
fi

if ! grep -q "generate_dr_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected DR evidence shared contract module to execute evidence bundle generator" >&2
  exit 1
fi

if ! grep -q "check_release_slo_gates.sh" "$SHARED_CONTRACT"; then
  echo "expected DR evidence shared contract module to execute release SLO policy checker" >&2
  exit 1
fi

if ! grep -Fq "run_dr_evidence_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute fast-lane DR checks first" >&2
  exit 1
fi

if ! grep -q "dr-evidence-report.json" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to emit DR evidence report artifact" >&2
  exit 1
fi

# Keep deployment SLO/rollback automation contract coverage on the deploy lane
# without widening workflow command count.
bash "$ROOT_DIR/scripts/deploy/test_run_deployment_slo_rollback_lane.sh"
bash "$ROOT_DIR/scripts/deploy/test_check_deployment_slo_rollback_policy.sh"
bash "$ROOT_DIR/scripts/deploy/test_run_deployment_slo_rollback_contract_lane.sh"

echo "dr evidence contract lane script tests passed."
