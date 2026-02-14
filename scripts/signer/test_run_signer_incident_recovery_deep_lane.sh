#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_incident_recovery_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/signer/run_signer_incident_recovery_deep_lane.sh"
DEEP_IMPL="$ROOT_DIR/scripts/signer/run_signer_incident_recovery_deep_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/signer_signer_incident_recovery_deep_lane.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected signer incident recovery lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected signer incident recovery deep lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$DEEP_IMPL" ]; then
  echo "expected signer incident recovery deep lane implementation to be executable" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi
if [ ! -L "$DEEP_LANE" ]; then
  echo "expected signer incident recovery deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$DEEP_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected signer incident recovery deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected signer incident recovery deep lane wrapper to resolve signer deep-lane manifest via dispatcher" >&2
  exit 1
fi
if ! grep -Fq "run_signer_incident_recovery_deep_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected signer incident recovery deep lane manifest to dispatch implementation module" >&2
  exit 1
fi

source_report="$TMP_DIR/signer-incident-recovery-source.json"
KAMN_SIGNER_INCIDENT_RECOVERY_SKIP_COMMANDS=true bash "$LANE_SCRIPT" --output-json "$source_report" >/dev/null

deep_report="$TMP_DIR/signer-incident-recovery-deep-summary.json"
set +e
unscheduled_output="$(
  bash "$DEEP_LANE" \
    --skip-contract-lane \
    --report-file "$source_report" \
    --output-json "$deep_report" 2>&1
)"
unscheduled_code=$?
set -e

if [ "$unscheduled_code" -eq 0 ]; then
  echo "expected signer incident recovery deep lane cadence guard to reject unscheduled execution" >&2
  exit 1
fi
if ! printf '%s\n' "$unscheduled_output" | grep -q "scheduled-only"; then
  echo "expected signer incident recovery deep lane cadence rejection marker" >&2
  exit 1
fi

scheduled_output="$(
  KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE=scheduled \
    bash "$DEEP_LANE" \
      --skip-contract-lane \
      --report-file "$source_report" \
      --output-json "$deep_report"
)"
if ! printf '%s\n' "$scheduled_output" | grep -q "signer incident recovery deep lane tests passed."; then
  echo "expected signer incident recovery deep lane success marker" >&2
  exit 1
fi

python3 - "$deep_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.signer.incident-recovery-deep-summary.v1":
    raise SystemExit("unexpected signer incident recovery deep report schema")
if payload.get("lane") != "deep":
    raise SystemExit("expected deep lane report")
if payload.get("status") != "pass":
    raise SystemExit("expected deep lane to pass under scheduled cadence")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected deep lane GO decision under scheduled cadence")
PY

set +e
stale_output="$(
  KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE=scheduled \
  KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_STALE_ARTIFACT=true \
    bash "$DEEP_LANE" \
      --skip-contract-lane \
      --report-file "$source_report" \
      --output-json "$deep_report" 2>&1
)"
stale_code=$?
set -e

if [ "$stale_code" -eq 0 ]; then
  echo "expected stale signer incident recovery deep artifact to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$stale_output" | grep -q "stale_deep_artifact"; then
  echo "expected stale deep artifact reason code marker" >&2
  exit 1
fi

echo "signer incident recovery deep lane script tests passed."
