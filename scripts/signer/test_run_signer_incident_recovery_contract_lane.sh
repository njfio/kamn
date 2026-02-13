#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/signer/run_signer_incident_recovery_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/signer/signer_incident_recovery_contract_lane_contract.py"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/signer_signer_incident_recovery_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected signer incident recovery contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected signer incident recovery shared contract-lane module to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/signer-incident-recovery-contract-report.json"
output="$(
  KAMN_SIGNER_INCIDENT_RECOVERY_CONTRACT_MAX_SECONDS=240 \
  KAMN_SIGNER_INCIDENT_RECOVERY_MAX_SECONDS=120 \
    bash "$SCRIPT" --output-file "$report_file"
)"

if ! printf '%s\n' "$output" | grep -q 'signer incident recovery contract lane tests passed.'; then
  echo "expected success output from signer incident recovery contract lane" >&2
  exit 1
fi

if [ ! -f "$report_file" ]; then
  echo "expected signer incident recovery contract lane to emit report file" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.signer.incident-recovery-report.v1"' "$report_file"; then
  echo "expected signer incident recovery report schema marker in contract lane output" >&2
  exit 1
fi

if ! grep -q '"final_decision": "GO"' "$report_file"; then
  echo "expected GO final decision in signer incident recovery contract lane report" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected signer incident recovery contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected signer incident recovery contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected signer incident recovery wrapper to resolve signer manifest via dispatcher" >&2
  exit 1
fi
if ! grep -Fq "signer_incident_recovery_contract_lane_contract.py" "$MANIFEST_FILE"; then
  echo "expected signer incident recovery manifest to dispatch to shared module" >&2
  exit 1
fi

if ! grep -q 'check_signer_incident_recovery_policy.sh' "$SHARED_CONTRACT"; then
  echo "expected signer incident recovery shared contract-lane module to execute policy checker" >&2
  exit 1
fi

if ! grep -q 'KAMN_SIGNER_INCIDENT_RECOVERY_CONTRACT_MAX_SECONDS' "$SHARED_CONTRACT"; then
  echo "expected signer incident recovery shared contract-lane module to enforce runtime guard env marker" >&2
  exit 1
fi

if ! grep -q 'KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_RUNBOOK_GAP' "$SHARED_CONTRACT"; then
  echo "expected signer incident recovery shared contract-lane module to cover forced runbook-gap path" >&2
  exit 1
fi

echo "signer incident recovery contract lane script tests passed."
