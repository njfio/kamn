#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_runtime_commit_live_evidence_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_runtime_commit_live_finality_evidence_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_runtime_commit_live_finality_evidence_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
FOUNDATION_DOC="$ROOT_DIR/docs/foundation/kolme-runtime-commit-client.md"
CI_STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local runtime-commit live finality evidence contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local runtime-commit live evidence policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local runtime-commit live finality evidence contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local runtime-commit live finality evidence contract lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/local_runtime_commit_live_finality_evidence_contract_lane.py",
]:
    raise SystemExit("expected runtime-commit live finality evidence manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected runtime-commit live finality evidence contract implementation to exist" >&2
  exit 1
fi

required_markers=(
  "run_local_runtime_commit_live_lane.sh"
  "check_local_runtime_commit_live_evidence_policy.py"
  "submit_evidence_marker_present"
  "finality_evidence_marker_present"
  "request_payload_evidence_marker_present"
  "request_payload_evidence_artifact_path"
  "submit_evidence_artifact_path"
  "finality_evidence_artifact_path"
  "request_finality_evidence_contract_version"
  "request_finality_evidence_linked"
  "request_payload_evidence_marker_missing"
  "finality_evidence_artifact_path_missing"
  "request_finality_evidence_linkage_missing"
  "finality_retry_contract_version"
  "finality_retry_max_attempts"
  "finality_retry_backoff_seconds"
  "finality_retry_attempts_used"
  "finality_retry_exhausted"
  "finality_retry_failure_class"
  "live_finality_retry_exhausted_timeout"
  "live_finality_retry_exhausted_failed"
  "finality_retry_failure_class_mismatch_for_timeout_reason"
  "finality_retry_attempts_used_mismatch_for_timeout_reason"
  "native_payload_pubkey_marker_present"
  "native_payload_nonce_marker_present"
  "native_payload_messages_marker_present"
  "Regression: #2099"
)
for marker in "${required_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected runtime-commit live finality evidence contract implementation marker: $marker" >&2
    exit 1
  fi
done

required_doc_markers=(
  "run_local_runtime_commit_live_finality_evidence_contract_lane.sh"
  "check_local_runtime_commit_live_evidence_policy.py"
  "submit_evidence_marker_present"
  "finality_evidence_marker_present"
  "request_payload_evidence_marker_present"
  "request_payload_evidence_artifact_path"
  "submit_evidence_artifact_path"
  "finality_evidence_artifact_path"
  "request_finality_evidence_contract_version"
  "request_finality_evidence_linked"
  "request_payload_evidence_marker_missing"
  "finality_evidence_artifact_path_missing"
  "request_finality_evidence_linkage_missing"
  "finality_retry_contract_version"
  "finality_retry_max_attempts"
  "finality_retry_backoff_seconds"
  "finality_retry_attempts_used"
  "finality_retry_exhausted"
  "finality_retry_failure_class"
  "live_finality_retry_exhausted_timeout"
  "live_finality_retry_exhausted_failed"
  "native_payload_pubkey_marker_present"
  "native_payload_nonce_marker_present"
  "native_payload_messages_marker_present"
)
for marker in "${required_doc_markers[@]}"; do
  if ! grep -q "$marker" "$DOC_FILE"; then
    echo "expected Kolme devnet ops documentation marker: $marker" >&2
    exit 1
  fi
  if ! grep -q "$marker" "$FOUNDATION_DOC"; then
    echo "expected runtime commit foundation documentation marker: $marker" >&2
    exit 1
  fi
  if ! grep -q "$marker" "$CI_STRATEGY_DOC"; then
    echo "expected CI strategy documentation marker: $marker" >&2
    exit 1
  fi
done

if ! grep -q "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local runtime-commit finality evidence contract lane" >&2
  exit 1
fi

bash "$RUNNER" --output-json "$TMP_REPORT" --policy-output-json "$TMP_POLICY_REPORT" >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))

if summary.get("schema_version") != "kamn.kolme.local-runtime-commit-live-summary.v1":
    raise SystemExit("unexpected runtime-commit live finality evidence summary schema")
if summary.get("provider_contract_enforcement_mode") != "live-provider-only-v1":
    raise SystemExit("expected provider_contract_enforcement_mode=live-provider-only-v1 in runtime-commit live finality evidence summary")
if summary.get("provider_live_contract_marker") != "provider_client_contract=KolmeRuntimeCommitLiveProvider":
    raise SystemExit("expected provider_live_contract_marker in runtime-commit live finality evidence summary")
if summary.get("provider_live_contract_marker_present") is not True:
    raise SystemExit("expected provider_live_contract_marker_present=true in runtime-commit live finality evidence summary")
if summary.get("provider_in_memory_reference_detected") is not False:
    raise SystemExit("expected provider_in_memory_reference_detected=false in runtime-commit live finality evidence summary")
if summary.get("provider_signer_adapter_contract") != "KolmeForkSecp256k1SignerAdapter":
    raise SystemExit("expected provider_signer_adapter_contract=KolmeForkSecp256k1SignerAdapter in runtime-commit live finality evidence summary")
if summary.get("provider_signing_curve_contract") != "secp256k1":
    raise SystemExit("expected provider_signing_curve_contract=secp256k1 in runtime-commit live finality evidence summary")
if summary.get("provider_signing_profile_contract_version") != "v1":
    raise SystemExit("expected provider_signing_profile_contract_version=v1 in runtime-commit live finality evidence summary")
if summary.get("status") != "ok":
    raise SystemExit("expected runtime-commit live finality evidence summary status ok")
if summary.get("reason_code") != "live_runtime_commit_and_finality_commands_passed":
    raise SystemExit("expected runtime-commit live finality evidence summary reason code")
if summary.get("finality_enabled") is not True:
    raise SystemExit("expected finality_enabled=true in runtime-commit live finality evidence summary")
if summary.get("submit_evidence_marker_present") is not True:
    raise SystemExit("expected submit_evidence_marker_present=true in runtime-commit live finality evidence summary")
if summary.get("finality_evidence_marker_present") is not True:
    raise SystemExit("expected finality_evidence_marker_present=true in runtime-commit live finality evidence summary")
if summary.get("request_payload_evidence_marker") != "native_payload_pubkey_nonce_messages":
    raise SystemExit("expected request_payload_evidence_marker in runtime-commit live finality evidence summary")
if summary.get("request_payload_evidence_marker_present") is not True:
    raise SystemExit("expected request_payload_evidence_marker_present=true in runtime-commit live finality evidence summary")
if summary.get("request_payload_evidence_artifact_path") != summary.get("live_output_file"):
    raise SystemExit("expected request_payload_evidence_artifact_path to match live_output_file in runtime-commit live finality evidence summary")
if summary.get("submit_evidence_artifact_path") != summary.get("live_output_file"):
    raise SystemExit("expected submit_evidence_artifact_path to match live_output_file in runtime-commit live finality evidence summary")
if summary.get("finality_evidence_artifact_path") != summary.get("finality_output_file"):
    raise SystemExit("expected finality_evidence_artifact_path to match finality_output_file in runtime-commit live finality evidence summary")
if summary.get("request_finality_evidence_contract_version") != "v1":
    raise SystemExit("expected request_finality_evidence_contract_version=v1 in runtime-commit live finality evidence summary")
if summary.get("request_finality_evidence_linked") is not True:
    raise SystemExit("expected request_finality_evidence_linked=true in runtime-commit live finality evidence summary")
if summary.get("finality_retry_contract_version") != "v1":
    raise SystemExit("expected finality_retry_contract_version=v1 in runtime-commit live finality evidence summary")
if summary.get("finality_retry_max_attempts") != 2:
    raise SystemExit("expected finality_retry_max_attempts=2 in runtime-commit live finality evidence summary")
if summary.get("finality_retry_backoff_seconds") != 0:
    raise SystemExit("expected finality_retry_backoff_seconds=0 in runtime-commit live finality evidence summary")
if summary.get("finality_retry_attempts_used") != 1:
    raise SystemExit("expected finality_retry_attempts_used=1 in runtime-commit live finality evidence summary")
if summary.get("finality_retry_exhausted") is not False:
    raise SystemExit("expected finality_retry_exhausted=false in runtime-commit live finality evidence summary")
if summary.get("finality_retry_failure_class") != "none":
    raise SystemExit("expected finality_retry_failure_class=none in runtime-commit live finality evidence summary")
if summary.get("native_payload_pubkey_marker_present") is not True:
    raise SystemExit("expected native_payload_pubkey_marker_present=true in runtime-commit live finality evidence summary")
if summary.get("native_payload_nonce_marker_present") is not True:
    raise SystemExit("expected native_payload_nonce_marker_present=true in runtime-commit live finality evidence summary")
if summary.get("native_payload_messages_marker_present") is not True:
    raise SystemExit("expected native_payload_messages_marker_present=true in runtime-commit live finality evidence summary")

if policy.get("schema_version") != "kamn.kolme.local-runtime-commit-live-policy-report.v1":
    raise SystemExit("unexpected runtime-commit live finality evidence policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected runtime-commit live finality evidence policy final_decision GO")
PY

TMP_LINKAGE_DRIFT_REPORT="$(mktemp)"
TMP_NEGATIVE_POLICY="$(mktemp)"
TMP_NEGATIVE_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT" "$TMP_LINKAGE_DRIFT_REPORT" "$TMP_NEGATIVE_POLICY" "$TMP_NEGATIVE_ERR"' EXIT

python3 - "$TMP_REPORT" "$TMP_LINKAGE_DRIFT_REPORT" <<'PY'
import json
import pathlib
import sys

base_summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
drift_summary = dict(base_summary)
drift_summary["request_finality_evidence_linked"] = False
drift_summary["finality_evidence_artifact_path"] = "/tmp/missing-runtime-finality-artifact.txt"
drift_summary["request_payload_evidence_marker_present"] = False
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(drift_summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_LINKAGE_DRIFT_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-native-payload-evidence \
  --output-json "$TMP_NEGATIVE_POLICY" >"$TMP_NEGATIVE_ERR" 2>&1
negative_exit_code=$?
set -e

if [ "$negative_exit_code" -eq 0 ]; then
  echo "expected request/finality linkage drift negative proof to fail closed" >&2
  exit 1
fi

if ! grep -q "request_finality_evidence_linkage_missing" "$TMP_NEGATIVE_ERR"; then
  echo "expected request_finality_evidence_linkage_missing reason in negative proof output" >&2
  exit 1
fi

if ! grep -q "finality_evidence_artifact_path_missing" "$TMP_NEGATIVE_ERR"; then
  echo "expected finality_evidence_artifact_path_missing reason in negative proof output" >&2
  exit 1
fi

if ! grep -q "request_payload_evidence_marker_missing" "$TMP_NEGATIVE_ERR"; then
  echo "expected request_payload_evidence_marker_missing reason in negative proof output" >&2
  exit 1
fi

TMP_RETRY_DRIFT_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT" "$TMP_LINKAGE_DRIFT_REPORT" "$TMP_NEGATIVE_POLICY" "$TMP_NEGATIVE_ERR" "$TMP_RETRY_DRIFT_REPORT"' EXIT

python3 - "$TMP_REPORT" "$TMP_RETRY_DRIFT_REPORT" <<'PY'
import json
import pathlib
import sys

base_summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
drift_summary = dict(base_summary)
drift_summary["status"] = "fail"
drift_summary["reason_code"] = "live_finality_retry_exhausted_timeout"
drift_summary["finality_retry_attempts_used"] = 2
drift_summary["finality_retry_exhausted"] = True
drift_summary["finality_evidence_marker_present"] = False
drift_summary["finality_retry_failure_class"] = "failed"
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(drift_summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_RETRY_DRIFT_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-reason-code live_finality_retry_exhausted_timeout \
  --output-json "$TMP_NEGATIVE_POLICY" >"$TMP_NEGATIVE_ERR" 2>&1
retry_negative_exit_code=$?
set -e

if [ "$retry_negative_exit_code" -eq 0 ]; then
  echo "expected retry failure-class drift proof to fail closed" >&2
  exit 1
fi

if ! grep -q "finality_retry_failure_class_mismatch_for_timeout_reason" "$TMP_NEGATIVE_ERR"; then
  echo "expected finality_retry_failure_class_mismatch_for_timeout_reason in retry drift output" >&2
  exit 1
fi

echo "local runtime-commit live finality evidence contract lane tests passed."
