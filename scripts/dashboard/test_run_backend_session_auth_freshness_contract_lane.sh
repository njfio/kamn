#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/dashboard/run_backend_session_auth_freshness_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/dashboard/backend_session_auth_freshness_contract_lane_contract.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/dashboard_backend_session_auth_freshness_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected dashboard backend session/auth freshness contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected dashboard backend session/auth freshness shared contract-lane module to be executable" >&2
  exit 1
fi
if [ ! -f "$MANIFEST" ]; then
  echo "expected dashboard backend session/auth freshness manifest to exist" >&2
  exit 1
fi

report_file="$TMP_DIR/dashboard-backend-session-auth-freshness-contract-report.json"
output="$(
  KAMN_DASHBOARD_BACKEND_SESSION_CONTRACT_MAX_SECONDS=240 \
  bash "$SCRIPT" --output-file "$report_file"
)"

if ! printf '%s\n' "$output" | grep -q 'dashboard backend session/auth freshness contract lane tests passed.'; then
  echo "expected success output from dashboard backend session/auth freshness contract lane" >&2
  exit 1
fi

if [ ! -f "$report_file" ]; then
  echo "expected dashboard backend session/auth freshness contract lane to emit report file" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.dashboard.backend-session-auth-freshness-report.v1"' "$report_file"; then
  echo "expected dashboard backend session/auth freshness report schema marker in contract lane output" >&2
  exit 1
fi

if ! grep -q '"final_decision": "GO"' "$report_file"; then
  echo "expected GO final decision in dashboard backend session/auth freshness contract lane report" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected dashboard backend session/auth freshness contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected dashboard backend session/auth freshness contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected dashboard backend session/auth freshness wrapper to resolve dashboard manifest via dispatcher" >&2
  exit 1
fi

if ! grep -q 'backend_session_auth_freshness_contract_lane_contract.py' "$MANIFEST"; then
  echo "expected dashboard backend session/auth freshness manifest to dispatch to shared module" >&2
  exit 1
fi

if ! grep -q 'check_backend_session_auth_freshness_policy.sh' "$SHARED_CONTRACT"; then
  echo "expected dashboard backend session/auth freshness shared contract-lane module to execute policy checker" >&2
  exit 1
fi

if ! grep -q 'KAMN_DASHBOARD_BACKEND_SESSION_CONTRACT_MAX_SECONDS' "$SHARED_CONTRACT"; then
  echo "expected dashboard backend session/auth freshness shared contract-lane module to enforce runtime guard env marker" >&2
  exit 1
fi

if ! grep -q 'KAMN_DASHBOARD_BACKEND_SESSION_FORCE_SESSION_GUARD_MISSING' "$SHARED_CONTRACT"; then
  echo "expected dashboard backend session/auth freshness shared contract-lane module to cover forced session-guard path" >&2
  exit 1
fi

echo "dashboard backend session/auth freshness contract lane script tests passed."
