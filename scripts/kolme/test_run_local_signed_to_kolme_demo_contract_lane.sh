#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_local_signed_to_kolme_demo_contract_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_signed_to_kolme_demo_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_signed_to_kolme_demo_contract_lane.py"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_signed_to_kolme_demo_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_DIR="$(mktemp -d)"
TMP_SUMMARY_DRY_RUN="$TMP_DIR/signed_to_kolme_demo_dry_run_summary.json"
TMP_POLICY_DRY_RUN="$TMP_DIR/signed_to_kolme_demo_dry_run_policy.json"
TMP_SUMMARY_RUN="$TMP_DIR/signed_to_kolme_demo_run_summary.json"
TMP_POLICY_RUN="$TMP_DIR/signed_to_kolme_demo_run_policy.json"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local signed-to-Kolme demo contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$CONTRACT_LANE"; then
  echo "expected local signed-to-Kolme demo contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local signed-to-Kolme demo contract lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("unexpected local signed-to-Kolme demo manifest schema")
if payload.get("lane_id") != "kolme.local_signed_to_kolme_demo.contract":
    raise SystemExit("unexpected local signed-to-Kolme demo manifest lane_id")
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/local_signed_to_kolme_demo_contract_lane.py",
]:
    raise SystemExit("unexpected local signed-to-Kolme demo manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local signed-to-Kolme demo contract implementation to exist" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local signed-to-Kolme demo policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_signed_to_kolme_demo_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local signed-to-Kolme demo contract lane" >&2
  exit 1
fi

if ! grep -q "check_local_signed_to_kolme_demo_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local signed-to-Kolme demo policy checker" >&2
  exit 1
fi

if ! grep -q "Regression: #1640" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local signed-to-Kolme demo regression marker" >&2
  exit 1
fi

if ! grep -q "Regression: #2388" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include runtime submit/finality regression marker" >&2
  exit 1
fi

if ! grep -q "run_local_signed_to_kolme_demo_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local signed-to-Kolme demo contract lane" >&2
  exit 1
fi

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --output-json "$TMP_SUMMARY_DRY_RUN" \
    --policy-output-json "$TMP_POLICY_DRY_RUN" \
    --max-seconds 120
)"
if ! printf '%s\n' "$lane_output" | grep -q "unified local signed-to-Kolme demo contract lane tests passed."; then
  echo "expected local signed-to-Kolme demo contract lane success marker" >&2
  exit 1
fi

python3 - "$TMP_SUMMARY_DRY_RUN" "$TMP_POLICY_DRY_RUN" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-signed-to-kolme-demo-summary.v1":
    raise SystemExit("unexpected signed-to-Kolme dry-run summary schema")
if summary.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode marker in signed-to-Kolme summary")
if summary.get("runtime_commit_submit_evidence_marker") != "status=submitted":
    raise SystemExit("expected runtime commit submit marker contract in signed-to-Kolme summary")
if summary.get("runtime_commit_finality_evidence_marker") != "finality=final":
    raise SystemExit("expected runtime commit finality marker contract in signed-to-Kolme summary")
if summary.get("runtime_commit_submit_evidence_marker_present") is not False:
    raise SystemExit("expected runtime commit submit marker absence in dry-run summary")
if summary.get("runtime_commit_finality_evidence_marker_present") is not False:
    raise SystemExit("expected runtime commit finality marker absence in dry-run summary")
if summary.get("runtime_commit_submit_finality_contract_version") != "v1":
    raise SystemExit("expected signed-to-Kolme submit/finality contract version marker")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected signed-to-Kolme dry-run policy final_decision GO")
PY

run_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$CONTRACT_LANE" \
      --mode run \
      --output-json "$TMP_SUMMARY_RUN" \
      --policy-output-json "$TMP_POLICY_RUN" \
      --max-seconds 240
)"
if ! printf '%s\n' "$run_output" | grep -q "unified local signed-to-Kolme demo contract lane tests passed."; then
  echo "expected signed-to-Kolme run mode success marker" >&2
  exit 1
fi

python3 - "$TMP_SUMMARY_RUN" "$TMP_POLICY_RUN" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if summary.get("mode") != "run":
    raise SystemExit("expected run mode marker in signed-to-Kolme summary")
if summary.get("status") != "ok":
    raise SystemExit("expected signed-to-Kolme run mode status=ok")
if summary.get("runtime_commit_submit_evidence_marker_present") is not True:
    raise SystemExit("expected runtime commit submit marker in run-mode signed-to-Kolme summary")
if summary.get("runtime_commit_finality_evidence_marker_present") is not True:
    raise SystemExit("expected runtime commit finality marker in run-mode signed-to-Kolme summary")
if summary.get("runtime_commit_submit_finality_linked") is not True:
    raise SystemExit("expected runtime commit submit/finality evidence linkage marker")
if summary.get("reason_code") != "signed_to_kolme_demo_passed":
    raise SystemExit("expected deterministic signed-to-Kolme pass reason code")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected signed-to-Kolme run policy final_decision GO")
PY

echo "local signed-to-Kolme demo contract lane tests passed."
