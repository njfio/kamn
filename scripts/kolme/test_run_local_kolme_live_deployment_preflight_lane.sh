#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_SUMMARY="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_SUMMARY" "$TMP_ERR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

if [ ! -x "$RUNNER" ]; then
  echo "expected local Kolme live deployment preflight lane runner to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_live_deployment_preflight_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference deployment preflight lane runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_live_deployment_preflight_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference deployment preflight policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_live_deployment_preflight_lane.sh" "$CI_DOC_FILE"; then
  echo "expected CI strategy doc to reference deployment preflight lane runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_live_deployment_preflight_policy.py" "$CI_DOC_FILE"; then
  echo "expected CI strategy doc to reference deployment preflight policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_live_deployment_preflight_lane.sh" "$README_FILE"; then
  echo "expected README to reference deployment preflight lane runner" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_live_deployment_preflight_policy.py" "$README_FILE"; then
  echo "expected README to reference deployment preflight policy checker" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --output-json "$TMP_SUMMARY"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected deployment preflight dry-run status"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected deployment preflight dry-run lane mode"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected deployment preflight dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "ci_fast_gate_eligible")" "true" "expected deployment preflight lane to be fast-gate eligible"

python3 - "$TMP_SUMMARY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-live-deployment-preflight-summary.v1":
    raise SystemExit("unexpected deployment preflight summary schema")
if summary.get("runtime_mode") != "kolme-live":
    raise SystemExit("expected runtime_mode=kolme-live in deployment preflight summary")
if summary.get("signer_profile_selector_env") != "KAMN_KOLME_LIVE_SIGNER_PROFILE":
    raise SystemExit("expected signer profile selector env marker in deployment preflight summary")
if summary.get("signer_profile") != "ops-primary":
    raise SystemExit("expected signer profile marker in deployment preflight summary")
if summary.get("signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX":
    raise SystemExit("expected signer private key env marker in deployment preflight summary")
if summary.get("fallback_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK":
    raise SystemExit("expected fallback signer private key env marker in deployment preflight summary")
if summary.get("fallback_signer_secret_present") is not False:
    raise SystemExit("expected fallback signer secret presence marker to be false in deployment preflight summary")
if summary.get("ci_fast_gate_eligible") is not True:
    raise SystemExit("expected deployment preflight summary to remain fast-gate eligible")
contracts = summary.get("contracts", {})
if contracts.get("ci_fast_gate_scope") != "ci-fast-gate":
    raise SystemExit("expected deployment preflight contracts to set ci-fast-gate scope")
if contracts.get("fallback_private_key_path_allowed") is not False:
    raise SystemExit("expected deployment preflight contracts to prohibit fallback private key paths")
PY

set +e
bash "$RUNNER" \
  --mode run \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
missing_secret_exit_code=$?
set -e

if [ "$missing_secret_exit_code" -eq 0 ]; then
  echo "expected deployment preflight run mode to fail closed when signer secret is missing" >&2
  exit 1
fi

if ! grep -q "signer secret env is required for selected profile" "$TMP_ERR"; then
  echo "expected deterministic missing signer secret message from deployment preflight lane" >&2
  exit 1
fi

set +e
KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX="1111111111111111111111111111111111111111111111111111111111111111" \
KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK="2222222222222222222222222222222222222222222222222222222222222222" \
bash "$RUNNER" \
  --mode run \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
fallback_secret_exit_code=$?
set -e

if [ "$fallback_secret_exit_code" -eq 0 ]; then
  echo "expected deployment preflight run mode to fail closed when fallback signer secret env is present" >&2
  exit 1
fi

if ! grep -q "fallback signer secret env must not be set" "$TMP_ERR"; then
  echo "expected deterministic fallback signer secret rejection message from deployment preflight lane" >&2
  exit 1
fi

KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX="1111111111111111111111111111111111111111111111111111111111111111" \
bash "$RUNNER" \
  --mode run \
  --output-json "$TMP_SUMMARY" >/dev/null

python3 - "$TMP_SUMMARY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if summary.get("status") != "ok":
    raise SystemExit("expected deployment preflight run summary status ok")
if summary.get("reason_code") != "deployment_preflight_passed":
    raise SystemExit("expected deployment preflight run summary pass reason code")
PY

echo "local Kolme live deployment preflight lane tests passed."
