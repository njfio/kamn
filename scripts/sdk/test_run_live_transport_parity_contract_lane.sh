#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/sdk/run_live_transport_parity_contract_lane.sh"
SHARED_FAST_SCRIPT="$ROOT_DIR/scripts/sdk/live_transport_parity_contract_lane_contract.py"
DEEP_SCRIPT="$ROOT_DIR/scripts/sdk/run_live_transport_parity_deep_lane.sh"
PROFILE_DRIFT_SCRIPT="$ROOT_DIR/scripts/sdk/run_transport_profile_parity_matrix.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/sdk_live_transport_parity_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected live transport parity fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected live transport parity contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected live transport parity contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected live transport parity wrapper to resolve sdk manifest via dispatcher" >&2
  exit 1
fi

if [ ! -x "$SHARED_FAST_SCRIPT" ]; then
  echo "expected shared live transport parity fast-lane implementation to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected live transport parity deep-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$PROFILE_DRIFT_SCRIPT" ]; then
  echo "expected transport profile parity drift matrix runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" --languages python,typescript >"$TMP_OUT"
if ! grep -q "live transport parity contract lane tests passed for languages: python,typescript." "$TMP_OUT"; then
  echo "expected live transport parity contract lane success marker" >&2
  exit 1
fi

if ! grep -q "running python live transport contract lane tests" "$TMP_OUT"; then
  echo "expected parity fast lane to run python subset tests" >&2
  exit 1
fi

if ! grep -q "running typescript live transport contract lane tests" "$TMP_OUT"; then
  echo "expected parity fast lane to run typescript subset tests" >&2
  exit 1
fi

if grep -q "running rust live transport contract lane tests" "$TMP_OUT"; then
  echo "expected parity fast lane python/typescript subset to skip rust tests" >&2
  exit 1
fi

if ! grep -q 'run_transport_profile_parity_matrix.sh' "$SHARED_FAST_SCRIPT"; then
  echo "expected shared parity fast-lane implementation to run transport profile parity drift matrix" >&2
  exit 1
fi

if ! grep -q 'live_transport_parity_contract_lane_contract.py' "$MANIFEST_FILE"; then
  echo "expected live transport parity manifest to dispatch shared contract module" >&2
  exit 1
fi

if ! grep -Fq "test_sdk_live_transport_deep.py" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute python deep-lane parity test" >&2
  exit 1
fi

if ! grep -Fq "live_transport_client.deep.ts" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute typescript deep-lane parity test" >&2
  exit 1
fi

echo "live transport parity contract lane script tests passed."
