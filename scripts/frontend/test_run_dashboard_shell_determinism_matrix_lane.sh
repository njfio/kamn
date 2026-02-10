#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/frontend/run_dashboard_shell_determinism_matrix_lane.sh"
SHARED_SCRIPT="$ROOT_DIR/scripts/frontend/dashboard_shell_determinism_matrix_lane_contract.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected dashboard shell determinism matrix lane script to be executable" >&2
  exit 1
fi

if ! grep -q 'dashboard_shell_determinism_matrix_lane_contract.py' "$SCRIPT"; then
  echo "expected dashboard shell matrix lane wrapper to delegate to shared implementation" >&2
  exit 1
fi

if [ ! -x "$SHARED_SCRIPT" ]; then
  echo "expected shared dashboard shell matrix lane implementation to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/dashboard-shell-matrix-go.json"
go_output="$(
  KAMN_FRONTEND_SHELL_MATRIX_SKIP_COMMANDS=true \
  bash "$SCRIPT" --output-json "$go_report"
)"

if ! printf '%s\n' "$go_output" | grep -q '^status=pass$'; then
  echo "expected dashboard shell matrix lane to report pass status for deterministic GO path" >&2
  exit 1
fi

if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  echo "expected dashboard shell matrix lane to report GO decision for deterministic GO path" >&2
  exit 1
fi

if ! printf '%s\n' "$go_output" | grep -q '^reason_codes=none$'; then
  echo "expected dashboard shell matrix lane to report no reason codes for deterministic GO path" >&2
  exit 1
fi

python3 - "$go_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.frontend.shell-matrix-report.v1":
    raise SystemExit("unexpected schema_version for dashboard shell matrix report")
if payload.get("status") != "pass":
    raise SystemExit("expected pass status for deterministic dashboard shell matrix GO path")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected GO final decision for deterministic dashboard shell matrix GO path")
if payload.get("reason_codes") != []:
    raise SystemExit("expected empty reason_codes for deterministic dashboard shell matrix GO path")
if payload.get("healthy_state_passed") is not True:
    raise SystemExit("expected healthy_state_passed=true for deterministic dashboard shell matrix GO path")
if payload.get("stale_critical_state_passed") is not True:
    raise SystemExit("expected stale_critical_state_passed=true for deterministic dashboard shell matrix GO path")
if payload.get("error_state_passed") is not True:
    raise SystemExit("expected error_state_passed=true for deterministic dashboard shell matrix GO path")
if payload.get("docs_contract_passed") is not True:
    raise SystemExit("expected docs_contract_passed=true for deterministic dashboard shell matrix GO path")
PY

set +e
stale_critical_no_go_output="$(
  KAMN_FRONTEND_SHELL_MATRIX_SKIP_COMMANDS=true \
  KAMN_FRONTEND_SHELL_MATRIX_FORCE_STALE_CRITICAL_STATE_MISSING=true \
  bash "$SCRIPT" --output-json "$TMP_DIR/dashboard-shell-matrix-no-go.json" 2>&1
)"
stale_critical_no_go_code=$?
set -e

if [ "$stale_critical_no_go_code" -eq 0 ]; then
  echo "expected forced stale/critical missing dashboard shell matrix lane run to fail closed" >&2
  exit 1
fi

if ! printf '%s\n' "$stale_critical_no_go_output" | grep -q 'stale_critical_state_missing'; then
  echo "expected forced stale/critical missing dashboard shell matrix lane run to emit stale_critical_state_missing reason code" >&2
  exit 1
fi

set +e
invalid_env_output="$(
  KAMN_FRONTEND_SHELL_MATRIX_MAX_SECONDS=invalid \
  bash "$SCRIPT" --output-json "$TMP_DIR/dashboard-shell-matrix-invalid-env.json" 2>&1
)"
invalid_env_code=$?
set -e

if [ "$invalid_env_code" -eq 0 ]; then
  echo "expected invalid KAMN_FRONTEND_SHELL_MATRIX_MAX_SECONDS to fail" >&2
  exit 1
fi

if ! printf '%s\n' "$invalid_env_output" | grep -q 'KAMN_FRONTEND_SHELL_MATRIX_MAX_SECONDS'; then
  echo "expected explicit validation output for invalid dashboard shell matrix max seconds env var" >&2
  exit 1
fi

echo "dashboard shell determinism matrix lane script tests passed."
