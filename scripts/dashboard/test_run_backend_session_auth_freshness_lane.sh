#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
SCRIPT="$ROOT_DIR/scripts/dashboard/run_backend_session_auth_freshness_lane.sh"
SCRIPT_IMPL="$ROOT_DIR/scripts/dashboard/run_backend_session_auth_freshness_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/dashboard_backend_session_auth_freshness_lane.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$SCRIPT" "expected dashboard backend session/auth freshness lane script to be executable"
test_harness_require_executable "$SCRIPT_IMPL" "expected dashboard backend session/auth freshness lane implementation to be executable"
test_harness_require_executable "$DISPATCHER" "expected shared non-Kolme dispatcher to be executable"
if [ ! -L "$SCRIPT" ]; then
  echo "expected dashboard backend session/auth freshness lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected dashboard backend session/auth freshness lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected dashboard backend session/auth freshness lane wrapper to resolve dashboard manifest via dispatcher" >&2
  exit 1
fi
if ! grep -Fq "run_backend_session_auth_freshness_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected dashboard backend session/auth freshness lane manifest to dispatch implementation module" >&2
  exit 1
fi

go_report="$TMP_DIR/dashboard-backend-session-auth-freshness-go.json"
go_output="$(
  KAMN_DASHBOARD_BACKEND_SESSION_SKIP_COMMANDS=true \
  bash "$SCRIPT" --output-json "$go_report"
)"

if ! printf '%s\n' "$go_output" | grep -q '^status=pass$'; then
  echo "expected dashboard backend session/auth freshness lane to report pass status for deterministic GO path" >&2
  exit 1
fi

if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  echo "expected dashboard backend session/auth freshness lane to report GO decision for deterministic GO path" >&2
  exit 1
fi

if ! printf '%s\n' "$go_output" | grep -q '^reason_codes=none$'; then
  echo "expected dashboard backend session/auth freshness lane to report no reason codes for deterministic GO path" >&2
  exit 1
fi

python3 - "$go_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.dashboard.backend-session-auth-freshness-report.v1":
    raise SystemExit("unexpected schema_version for dashboard backend session/auth freshness report")
if payload.get("status") != "pass":
    raise SystemExit("expected pass status for deterministic dashboard backend GO path")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected GO final decision for deterministic dashboard backend GO path")
if payload.get("reason_codes") != []:
    raise SystemExit("expected empty reason_codes for deterministic dashboard backend GO path")
if payload.get("session_guard_passed") is not True:
    raise SystemExit("expected session_guard_passed=true for deterministic dashboard backend GO path")
if payload.get("freshness_guard_passed") is not True:
    raise SystemExit("expected freshness_guard_passed=true for deterministic dashboard backend GO path")
if payload.get("docs_contract_passed") is not True:
    raise SystemExit("expected docs_contract_passed=true for deterministic dashboard backend GO path")
PY

set +e
session_no_go_output="$(
  KAMN_DASHBOARD_BACKEND_SESSION_SKIP_COMMANDS=true \
  KAMN_DASHBOARD_BACKEND_SESSION_FORCE_SESSION_GUARD_MISSING=true \
  bash "$SCRIPT" --output-json "$TMP_DIR/dashboard-backend-session-auth-freshness-session-no-go.json" 2>&1
)"
session_no_go_code=$?
set -e

if [ "$session_no_go_code" -eq 0 ]; then
  echo "expected forced session-guard-missing dashboard backend lane run to fail closed" >&2
  exit 1
fi

if ! printf '%s\n' "$session_no_go_output" | grep -q 'session_guard_missing'; then
  echo "expected forced session-guard-missing dashboard backend lane run to emit session_guard_missing reason code" >&2
  exit 1
fi

set +e
invalid_env_output="$(
  KAMN_DASHBOARD_BACKEND_SESSION_MAX_SECONDS=invalid \
  bash "$SCRIPT" --output-json "$TMP_DIR/dashboard-backend-session-auth-freshness-invalid-env.json" 2>&1
)"
invalid_env_code=$?
set -e

if [ "$invalid_env_code" -eq 0 ]; then
  echo "expected invalid KAMN_DASHBOARD_BACKEND_SESSION_MAX_SECONDS to fail" >&2
  exit 1
fi

if ! printf '%s\n' "$invalid_env_output" | grep -q 'KAMN_DASHBOARD_BACKEND_SESSION_MAX_SECONDS'; then
  echo "expected explicit validation output for invalid dashboard backend max seconds env var" >&2
  exit 1
fi

echo "dashboard backend session/auth freshness lane script tests passed."
