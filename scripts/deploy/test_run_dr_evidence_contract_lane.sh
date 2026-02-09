#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/deploy/run_dr_evidence_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/deploy/run_dr_evidence_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected DR evidence fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected DR evidence deep-lane runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "dr evidence contract lane tests passed." "$TMP_OUT"; then
  echo "expected DR evidence contract lane success marker" >&2
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
