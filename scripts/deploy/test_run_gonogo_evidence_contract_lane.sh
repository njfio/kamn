#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/deploy/run_gonogo_evidence_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/deploy/run_gonogo_evidence_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/deploy/gonogo_evidence_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/deploy_gonogo_evidence_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected go/no-go evidence fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected go/no-go evidence deep-lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected go/no-go evidence shared contract-lane module to be executable" >&2
  exit 1
fi

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$FAST_SCRIPT" >"$tmp_out"
if ! grep -q "go/no-go evidence contract lane tests passed." "$tmp_out"; then
  echo "expected go/no-go evidence contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected go/no-go evidence contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected go/no-go evidence contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected go/no-go evidence wrapper to resolve deploy manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "gonogo_evidence_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected go/no-go evidence manifest to dispatch shared contract module" >&2
  exit 1
fi

if ! grep -q "generate_gonogo_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected go/no-go evidence shared contract module to execute evidence bundle generator" >&2
  exit 1
fi

if ! grep -q "check_gonogo_evidence_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected go/no-go evidence shared contract module to execute policy checker" >&2
  exit 1
fi

if ! grep -q -- "--audit-integrity-report-file" "$SHARED_CONTRACT"; then
  echo "expected go/no-go evidence shared contract module to exercise audit-integrity gate arguments" >&2
  exit 1
fi

if ! grep -q -- "--slo-policy-report-file" "$SHARED_CONTRACT"; then
  echo "expected go/no-go evidence shared contract module to exercise SLO policy gate arguments" >&2
  exit 1
fi

if ! grep -Fq "run_gonogo_evidence_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute fast-lane contract checks first" >&2
  exit 1
fi

if ! grep -q "final_decision=NO-GO" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to validate NO-GO decision path" >&2
  exit 1
fi

echo "go/no-go evidence contract lane script tests passed."
