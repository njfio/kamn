#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/deploy/run_staging_rehearsal_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/deploy/run_staging_rehearsal_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/deploy/staging_rehearsal_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/deploy_staging_rehearsal_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected staging rehearsal fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected staging rehearsal deep-lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected staging rehearsal shared contract-lane module to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "staging rehearsal contract lane tests passed." "$TMP_OUT"; then
  echo "expected staging rehearsal contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected staging rehearsal contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected staging rehearsal contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected staging rehearsal wrapper to resolve deploy manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "staging_rehearsal_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected staging rehearsal manifest to dispatch shared contract module" >&2
  exit 1
fi

if ! grep -Fq "run_staging_rehearsal_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute fast-lane rehearsal checks first" >&2
  exit 1
fi

if ! grep -q "staging-rehearsal-report.json" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to emit staging rehearsal report artifact" >&2
  exit 1
fi

if ! grep -Fq -- "--recovery-time-seconds" "$SHARED_CONTRACT"; then
  echo "expected staging rehearsal shared contract module to pass deterministic MTTR evidence markers" >&2
  exit 1
fi

if ! grep -Fq -- "--max-allowed-recovery-time-seconds" "$SHARED_CONTRACT"; then
  echo "expected staging rehearsal shared contract module to pass bounded MTTR contract markers" >&2
  exit 1
fi

if ! grep -Fq "staged_rehearsal_signoff_status=verified" "$SHARED_CONTRACT"; then
  echo "expected staging rehearsal shared contract module to assert verified staged signoff status" >&2
  exit 1
fi

if ! grep -Fq -- "--recovery-time-seconds" "$DEEP_SCRIPT"; then
  echo "expected staging rehearsal deep-lane runner to pass deterministic MTTR evidence markers" >&2
  exit 1
fi

if ! grep -Fq -- "--max-allowed-recovery-time-seconds" "$DEEP_SCRIPT"; then
  echo "expected staging rehearsal deep-lane runner to pass bounded MTTR contract markers" >&2
  exit 1
fi

if ! grep -Fq "staged_rehearsal_signoff_status=fail-closed" "$DEEP_SCRIPT"; then
  echo "expected staging rehearsal deep-lane runner to assert fail-closed staged signoff status" >&2
  exit 1
fi

echo "staging rehearsal contract lane script tests passed."
