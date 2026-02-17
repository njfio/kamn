#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_local_signed_to_kolme_demo_contract_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_signed_to_kolme_demo_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_signed_to_kolme_demo_contract_lane.py"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_signed_to_kolme_demo_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
RELEASE_GONOGO_DOC="$ROOT_DIR/docs/foundation/release-gonogo-checklist.md"
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

if [ ! -f "$RELEASE_GONOGO_DOC" ]; then
  echo "expected release go/no-go checklist doc to exist" >&2
  exit 1
fi

if ! grep -q "check_local_signed_to_kolme_demo_policy.py" "$RELEASE_GONOGO_DOC"; then
  echo "expected release go/no-go checklist doc to reference signed-to-Kolme demo policy checker" >&2
  exit 1
fi

if ! grep -q "signed_message_commit_evidence_mismatch" "$RELEASE_GONOGO_DOC"; then
  echo "expected release go/no-go checklist doc to include signed-message/commit mismatch marker" >&2
  exit 1
fi

if ! grep -q "Regression: #4497" "$RELEASE_GONOGO_DOC"; then
  echo "expected release go/no-go checklist doc to include signed-message/commit mismatch regression marker" >&2
  exit 1
fi

if ! grep -q "demo_evidence_reason_taxonomy_version=kamn.kolme.local-signed-to-kolme-demo.reason-taxonomy.v1" "$RELEASE_GONOGO_DOC"; then
  echo "expected release go/no-go checklist doc to include signed-to-Kolme reason taxonomy marker" >&2
  exit 1
fi

if ! grep -q "demo_evidence_normalization_version=kamn.kolme.local-signed-to-kolme-demo.evidence-normalization.v1" "$RELEASE_GONOGO_DOC"; then
  echo "expected release go/no-go checklist doc to include signed-to-Kolme normalization marker" >&2
  exit 1
fi

if ! grep -q "native_signer_reason_taxonomy_version=kamn.kolme.local-signed-to-kolme-demo-native-signer-reason-taxonomy.v1" "$RELEASE_GONOGO_DOC"; then
  echo "expected release go/no-go checklist doc to include signed-to-Kolme native signer taxonomy marker" >&2
  exit 1
fi

if ! grep -q "runtime_commit_native_signing_profile_marker_missing" "$RELEASE_GONOGO_DOC"; then
  echo "expected release go/no-go checklist doc to include signed-to-Kolme native signer marker-missing reason" >&2
  exit 1
fi

if ! grep -q "runtime_signing_profile=kolme-fork-secp256k1-v1" "$RELEASE_GONOGO_DOC"; then
  echo "expected release go/no-go checklist doc to include signed-to-Kolme native runtime signing profile marker" >&2
  exit 1
fi

if ! grep -q "Regression: #4373" "$RELEASE_GONOGO_DOC"; then
  echo "expected release go/no-go checklist doc to include native signer rejection regression marker" >&2
  exit 1
fi

if ! grep -q "Regression: #4380" "$RELEASE_GONOGO_DOC"; then
  echo "expected release go/no-go checklist doc to include native signer taxonomy regression marker" >&2
  exit 1
fi

if ! grep -q "Regression: #4498" "$RELEASE_GONOGO_DOC"; then
  echo "expected release go/no-go checklist doc to include signed-to-Kolme taxonomy normalization regression marker" >&2
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
if summary.get("runtime_signing_profile_contract_version") != "v1":
    raise SystemExit("expected signed-to-Kolme runtime signing profile contract version marker")
if summary.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
    raise SystemExit("expected signed-to-Kolme runtime signing profile marker")
reason_taxonomy = summary.get("reason_taxonomy")
if not isinstance(reason_taxonomy, dict):
    raise SystemExit("expected signed-to-Kolme dry-run reason taxonomy")
if reason_taxonomy.get("schema_version") != "kamn.kolme.local-signed-to-kolme-demo.reason-taxonomy.v1":
    raise SystemExit("unexpected signed-to-Kolme dry-run reason taxonomy schema")
if reason_taxonomy.get("overall") != "demo.not_run":
    raise SystemExit("expected signed-to-Kolme dry-run overall taxonomy classification")
if reason_taxonomy.get("signed_demo_checkpoint") != "checkpoint.planned":
    raise SystemExit("expected signed-to-Kolme dry-run signed-demo checkpoint taxonomy classification")
if reason_taxonomy.get("signed_integration_checkpoint") != "checkpoint.planned":
    raise SystemExit("expected signed-to-Kolme dry-run signed-integration checkpoint taxonomy classification")
if reason_taxonomy.get("runtime_integration_checkpoint") != "checkpoint.planned":
    raise SystemExit("expected signed-to-Kolme dry-run runtime-integration checkpoint taxonomy classification")
if reason_taxonomy.get("runtime_commit_live") != "runtime_commit.not_run":
    raise SystemExit("expected signed-to-Kolme dry-run runtime-commit taxonomy classification")
normalized_evidence = summary.get("normalized_evidence")
if not isinstance(normalized_evidence, dict):
    raise SystemExit("expected signed-to-Kolme dry-run normalized evidence")
if normalized_evidence.get("schema_version") != "kamn.kolme.local-signed-to-kolme-demo.evidence-normalization.v1":
    raise SystemExit("unexpected signed-to-Kolme dry-run normalized evidence schema")
expected_order = [
    "localhost_signed_demo_contract",
    "localhost_signed_integration_contract",
    "local_kamn_runtime_integration_run",
]
if normalized_evidence.get("primary_check_order") != expected_order:
    raise SystemExit("unexpected signed-to-Kolme dry-run normalized primary check order")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected signed-to-Kolme dry-run policy final_decision GO")
if policy.get("native_signer_reason_taxonomy_version") != "kamn.kolme.local-signed-to-kolme-demo-native-signer-reason-taxonomy.v1":
    raise SystemExit("expected signed-to-Kolme dry-run native signer taxonomy version marker")
if policy.get("native_signer_reason_codes_csv") != "runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch":
    raise SystemExit("expected signed-to-Kolme dry-run native signer reason csv marker")
if policy.get("native_signer_reason_codes_value") != "none":
    raise SystemExit("expected signed-to-Kolme dry-run native signer reason value marker")
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
if summary.get("runtime_signing_profile_contract_version") != "v1":
    raise SystemExit("expected signed-to-Kolme run runtime signing profile contract version marker")
if summary.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
    raise SystemExit("expected signed-to-Kolme run runtime signing profile marker")
if summary.get("reason_code") != "signed_to_kolme_demo_passed":
    raise SystemExit("expected deterministic signed-to-Kolme pass reason code")
reason_taxonomy = summary.get("reason_taxonomy")
if not isinstance(reason_taxonomy, dict):
    raise SystemExit("expected signed-to-Kolme run reason taxonomy")
if reason_taxonomy.get("schema_version") != "kamn.kolme.local-signed-to-kolme-demo.reason-taxonomy.v1":
    raise SystemExit("unexpected signed-to-Kolme run reason taxonomy schema")
if reason_taxonomy.get("overall") != "demo.success":
    raise SystemExit("expected signed-to-Kolme run overall taxonomy classification")
if reason_taxonomy.get("signed_demo_checkpoint") != "checkpoint.pass":
    raise SystemExit("expected signed-to-Kolme run signed-demo checkpoint taxonomy classification")
if reason_taxonomy.get("signed_integration_checkpoint") != "checkpoint.pass":
    raise SystemExit("expected signed-to-Kolme run signed-integration checkpoint taxonomy classification")
if reason_taxonomy.get("runtime_integration_checkpoint") != "checkpoint.pass":
    raise SystemExit("expected signed-to-Kolme run runtime-integration checkpoint taxonomy classification")
if reason_taxonomy.get("runtime_commit_live") != "runtime_commit.success":
    raise SystemExit("expected signed-to-Kolme run runtime-commit taxonomy classification")
normalized_evidence = summary.get("normalized_evidence")
if not isinstance(normalized_evidence, dict):
    raise SystemExit("expected signed-to-Kolme run normalized evidence")
if normalized_evidence.get("schema_version") != "kamn.kolme.local-signed-to-kolme-demo.evidence-normalization.v1":
    raise SystemExit("unexpected signed-to-Kolme run normalized evidence schema")
checks_by_id = normalized_evidence.get("checks_by_id")
if not isinstance(checks_by_id, dict):
    raise SystemExit("expected signed-to-Kolme run normalized checks map")
runtime_entry = checks_by_id.get("local_kamn_runtime_integration_run")
if not isinstance(runtime_entry, dict):
    raise SystemExit("expected signed-to-Kolme run normalized runtime integration entry")
if runtime_entry.get("status") != "pass":
    raise SystemExit("expected signed-to-Kolme run normalized runtime integration pass status")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected signed-to-Kolme run policy final_decision GO")
if policy.get("native_signer_reason_taxonomy_version") != "kamn.kolme.local-signed-to-kolme-demo-native-signer-reason-taxonomy.v1":
    raise SystemExit("expected signed-to-Kolme run native signer taxonomy version marker")
if policy.get("native_signer_reason_codes_csv") != "runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch":
    raise SystemExit("expected signed-to-Kolme run native signer reason csv marker")
if policy.get("native_signer_reason_codes_value") != "none":
    raise SystemExit("expected signed-to-Kolme run native signer reason value marker")
PY

# Regression: #4497
TMP_MISMATCH_SIGNED_CHECK="$TMP_DIR/signed_to_kolme_demo_mismatch_signed_check.json"
TMP_MISMATCH_INTEGRATION_CHECK="$TMP_DIR/signed_to_kolme_demo_mismatch_integration_check.json"
TMP_MISMATCH_POLICY="$TMP_DIR/signed_to_kolme_demo_mismatch_policy.json"
TMP_MISMATCH_ERR="$TMP_DIR/signed_to_kolme_demo_mismatch.err"
TMP_TAXONOMY_DRIFT="$TMP_DIR/signed_to_kolme_demo_taxonomy_drift.json"
TMP_NORMALIZED_DRIFT="$TMP_DIR/signed_to_kolme_demo_normalized_drift.json"

python3 - "$TMP_SUMMARY_RUN" "$TMP_MISMATCH_SIGNED_CHECK" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
summary["mode"] = "run"
summary["status"] = "ok"
summary["reason_code"] = "signed_to_kolme_demo_passed"
for check in summary.get("checks", []):
    if not isinstance(check, dict):
        continue
    if check.get("id") == "localhost_signed_demo_contract":
        check["status"] = "fail"
        check["reason_code"] = "localhost_signed_demo_contract_failed"
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_MISMATCH_SIGNED_CHECK" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_MISMATCH_POLICY" >"$TMP_MISMATCH_ERR" 2>&1
mismatch_signed_check_code=$?
set -e
if [ "$mismatch_signed_check_code" -eq 0 ]; then
  echo "expected checker to fail when signed-message checkpoint failure is accepted alongside commit evidence success" >&2
  exit 1
fi
if ! grep -q "signed_message_commit_evidence_mismatch" "$TMP_MISMATCH_ERR"; then
  echo "expected deterministic signed_message_commit_evidence_mismatch marker for signed-check mismatch" >&2
  exit 1
fi

python3 - "$TMP_SUMMARY_RUN" "$TMP_MISMATCH_INTEGRATION_CHECK" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
summary["mode"] = "run"
summary["status"] = "ok"
summary["reason_code"] = "signed_to_kolme_demo_passed"
for check in summary.get("checks", []):
    if not isinstance(check, dict):
        continue
    if check.get("id") == "local_kamn_runtime_integration_run":
        check["status"] = "fail"
        check["reason_code"] = "local_kamn_runtime_integration_run_failed"
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_MISMATCH_INTEGRATION_CHECK" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_MISMATCH_POLICY" >"$TMP_MISMATCH_ERR" 2>&1
mismatch_integration_check_code=$?
set -e
if [ "$mismatch_integration_check_code" -eq 0 ]; then
  echo "expected checker to fail when integration checkpoint failure is accepted alongside commit evidence success" >&2
  exit 1
fi
if ! grep -q "signed_message_commit_evidence_mismatch" "$TMP_MISMATCH_ERR"; then
  echo "expected deterministic signed_message_commit_evidence_mismatch marker for integration-check mismatch" >&2
  exit 1
fi

# Regression: #4498
python3 - "$TMP_SUMMARY_RUN" "$TMP_TAXONOMY_DRIFT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
summary["reason_taxonomy"]["overall"] = "demo.not_run"
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_TAXONOMY_DRIFT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_MISMATCH_POLICY" >"$TMP_MISMATCH_ERR" 2>&1
taxonomy_drift_code=$?
set -e
if [ "$taxonomy_drift_code" -eq 0 ]; then
  echo "expected checker to fail when signed-to-Kolme taxonomy output drifts" >&2
  exit 1
fi
if ! grep -q "reason_taxonomy_overall_mismatch" "$TMP_MISMATCH_ERR"; then
  echo "expected reason_taxonomy_overall_mismatch marker for signed-to-Kolme taxonomy drift" >&2
  exit 1
fi

python3 - "$TMP_SUMMARY_RUN" "$TMP_NORMALIZED_DRIFT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
summary["normalized_evidence"]["checks_by_id"]["local_kamn_runtime_integration_run"]["status"] = "fail"
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_NORMALIZED_DRIFT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_MISMATCH_POLICY" >"$TMP_MISMATCH_ERR" 2>&1
normalized_drift_code=$?
set -e
if [ "$normalized_drift_code" -eq 0 ]; then
  echo "expected checker to fail when signed-to-Kolme normalized evidence output drifts" >&2
  exit 1
fi
if ! grep -q "normalized_evidence_status_mismatch:local_kamn_runtime_integration_run" "$TMP_MISMATCH_ERR"; then
  echo "expected normalized_evidence_status_mismatch marker for signed-to-Kolme normalized evidence drift" >&2
  exit 1
fi

echo "local signed-to-Kolme demo contract lane tests passed."
