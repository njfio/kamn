#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/deploy/run_staging_rehearsal_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/deploy/run_staging_rehearsal_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/deploy/staging_rehearsal_contract_lane_contract.sh"
SHARED_REHEARSAL_CONTRACT_PY="$ROOT_DIR/scripts/deploy/staging_rehearsal_contract.py"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/deploy_staging_rehearsal_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
INCIDENT_READINESS_DOC="$ROOT_DIR/docs/ops/incident-readiness.md"

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

if [ ! -f "$INCIDENT_READINESS_DOC" ]; then
  echo "expected incident readiness ops doc to exist" >&2
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

if ! grep -Fq "command_contracts" "$SHARED_REHEARSAL_CONTRACT_PY"; then
  echo "expected staging rehearsal contract generator to emit command contract markers" >&2
  exit 1
fi

if ! grep -Fq "evidence_output_contract_version" "$SHARED_REHEARSAL_CONTRACT_PY"; then
  echo "expected staging rehearsal contract generator to emit evidence output contract version marker" >&2
  exit 1
fi

if ! grep -Fq "reason_taxonomy" "$SHARED_REHEARSAL_CONTRACT_PY"; then
  echo "expected staging rehearsal contract generator to emit reason taxonomy output" >&2
  exit 1
fi

if ! grep -Fq "normalized_evidence" "$SHARED_REHEARSAL_CONTRACT_PY"; then
  echo "expected staging rehearsal contract generator to emit normalized evidence output" >&2
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

if ! grep -Fq "generate_staging_rehearsal_bundle.sh" "$INCIDENT_READINESS_DOC"; then
  echo "expected incident readiness ops doc to include rehearsal bundle generator command" >&2
  exit 1
fi

if ! grep -Fq "check_staging_rehearsal_policy.sh" "$INCIDENT_READINESS_DOC"; then
  echo "expected incident readiness ops doc to include rehearsal policy checker command" >&2
  exit 1
fi

if ! grep -Fq "run_staging_rehearsal_contract_lane.sh" "$INCIDENT_READINESS_DOC"; then
  echo "expected incident readiness ops doc to include rehearsal contract lane command" >&2
  exit 1
fi

if ! grep -Fq "run_staging_rehearsal_deep_lane.sh" "$INCIDENT_READINESS_DOC"; then
  echo "expected incident readiness ops doc to include rehearsal deep lane command" >&2
  exit 1
fi

if ! grep -Fq "command contract mismatch" "$INCIDENT_READINESS_DOC"; then
  echo "expected incident readiness ops doc to document command-contract drift guard" >&2
  exit 1
fi

if ! grep -Fq "evidence output contract version mismatch" "$INCIDENT_READINESS_DOC"; then
  echo "expected incident readiness ops doc to document evidence-output contract drift guard" >&2
  exit 1
fi

if ! grep -Fq "rehearsal_reason_taxonomy_version=kamn.release.staging-rehearsal-reason-taxonomy.v1" "$INCIDENT_READINESS_DOC"; then
  echo "expected incident readiness ops doc to document rehearsal reason taxonomy version marker" >&2
  exit 1
fi

if ! grep -Fq "rehearsal_normalized_evidence_version=kamn.release.staging-rehearsal-evidence-normalization.v1" "$INCIDENT_READINESS_DOC"; then
  echo "expected incident readiness ops doc to document rehearsal normalized evidence version marker" >&2
  exit 1
fi

if ! grep -Fq "reason taxonomy mismatch" "$INCIDENT_READINESS_DOC"; then
  echo "expected incident readiness ops doc to document reason-taxonomy drift guard" >&2
  exit 1
fi

if ! grep -Fq "normalized evidence bundle mismatch" "$INCIDENT_READINESS_DOC"; then
  echo "expected incident readiness ops doc to document normalized-evidence drift guard" >&2
  exit 1
fi

if ! grep -Fq "Regression: #4499" "$INCIDENT_READINESS_DOC"; then
  echo "expected incident readiness ops doc to include rehearsal drift regression marker" >&2
  exit 1
fi

if ! grep -Fq "Regression: #4500" "$INCIDENT_READINESS_DOC"; then
  echo "expected incident readiness ops doc to include rehearsal taxonomy normalization regression marker" >&2
  exit 1
fi

echo "staging rehearsal contract lane script tests passed."
