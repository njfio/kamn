#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/deploy/run_deployment_slo_rollback_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected deployment slo/rollback lane script to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/deployment-slo-rollback-go.json"
go_output="$(
  KAMN_DEPLOYMENT_SLO_ROLLBACK_SKIP_COMMANDS=true \
  bash "$SCRIPT" --output-json "$go_report"
)"

if ! printf '%s\n' "$go_output" | grep -q '^status=pass$'; then
  echo "expected deployment slo/rollback lane to report pass status for deterministic GO path" >&2
  exit 1
fi

if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  echo "expected deployment slo/rollback lane to report GO decision for deterministic GO path" >&2
  exit 1
fi

if ! printf '%s\n' "$go_output" | grep -q '^reason_codes=none$'; then
  echo "expected deployment slo/rollback lane to report no reason codes for deterministic GO path" >&2
  exit 1
fi

python3 - "$go_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.deploy.slo-rollback-report.v1":
    raise SystemExit("unexpected schema_version for deployment slo/rollback report")
if payload.get("status") != "pass":
    raise SystemExit("expected pass status for deterministic deployment slo/rollback GO path")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected GO final decision for deterministic deployment slo/rollback GO path")
if payload.get("reason_codes") != []:
    raise SystemExit("expected empty reason_codes for deterministic deployment slo/rollback GO path")
if payload.get("rollback_automation_passed") is not True:
    raise SystemExit("expected rollback_automation_passed=true for deterministic deployment slo/rollback GO path")
if payload.get("slo_gate_passed") is not True:
    raise SystemExit("expected slo_gate_passed=true for deterministic deployment slo/rollback GO path")
if payload.get("docs_contract_passed") is not True:
    raise SystemExit("expected docs_contract_passed=true for deterministic deployment slo/rollback GO path")
PY

set +e
rollback_no_go_output="$(
  KAMN_DEPLOYMENT_SLO_ROLLBACK_SKIP_COMMANDS=true \
  KAMN_DEPLOYMENT_SLO_ROLLBACK_FORCE_ROLLBACK_AUTOMATION_MISSING=true \
  bash "$SCRIPT" --output-json "$TMP_DIR/deployment-slo-rollback-no-go.json" 2>&1
)"
rollback_no_go_code=$?
set -e

if [ "$rollback_no_go_code" -eq 0 ]; then
  echo "expected forced rollback automation missing deployment slo/rollback lane run to fail closed" >&2
  exit 1
fi

if ! printf '%s\n' "$rollback_no_go_output" | grep -q 'rollback_automation_missing'; then
  echo "expected forced rollback automation missing deployment slo/rollback lane run to emit rollback_automation_missing reason code" >&2
  exit 1
fi

set +e
invalid_env_output="$(
  KAMN_DEPLOYMENT_SLO_ROLLBACK_MAX_SECONDS=invalid \
  bash "$SCRIPT" --output-json "$TMP_DIR/deployment-slo-rollback-invalid-env.json" 2>&1
)"
invalid_env_code=$?
set -e

if [ "$invalid_env_code" -eq 0 ]; then
  echo "expected invalid KAMN_DEPLOYMENT_SLO_ROLLBACK_MAX_SECONDS to fail" >&2
  exit 1
fi

if ! printf '%s\n' "$invalid_env_output" | grep -q 'KAMN_DEPLOYMENT_SLO_ROLLBACK_MAX_SECONDS'; then
  echo "expected explicit validation output for invalid deployment slo/rollback max seconds env var" >&2
  exit 1
fi

echo "deployment slo/rollback lane script tests passed."
