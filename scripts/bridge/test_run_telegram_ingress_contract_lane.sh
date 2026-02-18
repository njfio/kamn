#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
FAST_SCRIPT="$ROOT_DIR/scripts/bridge/run_telegram_ingress_contract_lane.sh"
FAST_IMPL_SCRIPT="$ROOT_DIR/scripts/bridge/run_telegram_ingress_contract_lane_impl.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/bridge/run_telegram_ingress_deep_lane.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_telegram_ingress_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

test_harness_require_executable "$FAST_SCRIPT" "expected telegram ingress fast-lane runner to be executable"

test_harness_require_executable "$DEEP_SCRIPT" "expected telegram ingress deep-lane runner to be executable"

test_harness_require_executable "$FAST_IMPL_SCRIPT" "expected telegram ingress contract lane implementation script to be executable"

test_harness_require_file "$MANIFEST_FILE" "expected telegram ingress contract lane manifest to exist"

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" --skip-replay >"$TMP_OUT"
if ! grep -q "telegram ingress contract lane tests passed." "$TMP_OUT"; then
  echo "expected telegram ingress contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_telegram_ingress_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute fast-lane telegram checks first" >&2
  exit 1
fi

if ! grep -q "telegram-ingress-report.json" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to emit telegram ingress report artifact" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected telegram ingress contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected telegram ingress contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected telegram ingress wrapper to resolve telegram manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_telegram_ingress_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected telegram ingress manifest to dispatch to implementation script" >&2
  exit 1
fi

if ! grep -q "run_bridge_replay_matrix.sh" "$FAST_IMPL_SCRIPT"; then
  echo "expected telegram ingress implementation script to execute bridge replay matrix" >&2
  exit 1
fi

echo "telegram ingress contract lane script tests passed."
