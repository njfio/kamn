#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/cutover/generate_cutover_rollback_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/cutover/check_cutover_rollback_evidence_policy.sh"
NEXT_STEPS_DOC="$KAMN_ROOT/docs/plans/2026-02-14-production-service-next-steps.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected cutover rollback evidence bundle generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected cutover rollback evidence policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$NEXT_STEPS_DOC" ]; then
  echo "expected production-service next-steps docs file for cutover rollback docs-contract checks" >&2
  exit 1
fi

check_cutover_rollback_docs_contract() {
  local docs_file="$1"
  python3 - "$docs_file" <<'PY'
import pathlib
import sys

doc_path = pathlib.Path(sys.argv[1])
doc_text = doc_path.read_text(encoding="utf-8")
required_markers = {
    "cutover_rollback_schema_version=kamn.cutover.rollback-evidence.v1",
    "cutover_rollback_summary_markers=final_decision,rollback_hash_match,evidence_complete",
    "cutover_rollback_checkpoint_markers=rollback.trigger_status,rollback.checkpoint_state,rollback.failed_checkpoint_id",
    "cutover_rollback_reason_codes_csv=ci-fast-gate-failed,incomplete-evidence,rollback target hash mismatch,missing failed checkpoint evidence,trigger-state-checkpoint-mismatch,clear-trigger-requires-ready-checkpoint",
}
missing = sorted(marker for marker in required_markers if marker not in doc_text)
if missing:
    raise SystemExit(f"cutover_rollback_docs_missing_marker:{missing[0]}")
PY
}

go_bundle="$TMP_DIR/rollback-go.json"
go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --cutover-manifest-id "cutover-mainnet-2026-02-09" \
    --rollback-trigger-status CLEAR \
    --checkpoint-state READY \
    --failed-checkpoint-id "" \
    --rollback-target-hash "state-hash-abc" \
    --post-rollback-hash "state-hash-abc" \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_generate_output" "status")" "generated" "expected GO rollback bundle generation to succeed"
assert_eq "$(extract_value "$go_generate_output" "final_decision")" "GO" "expected generator to derive GO rollback decision"
check_cutover_rollback_docs_contract "$NEXT_STEPS_DOC"

python3 - "$go_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.cutover.rollback-evidence.v1":
    raise SystemExit("expected deterministic cutover rollback schema_version marker")
rollback = payload.get("rollback")
if not isinstance(rollback, dict):
    raise SystemExit("expected rollback object in cutover rollback evidence bundle")
if rollback.get("trigger_status") != "CLEAR":
    raise SystemExit("expected rollback.trigger_status=CLEAR for GO path")
if rollback.get("checkpoint_state") != "READY":
    raise SystemExit("expected rollback.checkpoint_state=READY for GO path")
if rollback.get("failed_checkpoint_id") is not None:
    raise SystemExit("expected rollback.failed_checkpoint_id=null for GO path")
if payload.get("decision_reasons") != []:
    raise SystemExit("expected decision_reasons=[] for GO path")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO for GO path")
PY

docs_drift_file="$TMP_DIR/production-service-next-steps.cutover-rollback-docs-drift.md"
cp "$NEXT_STEPS_DOC" "$docs_drift_file"
python3 - "$docs_drift_file" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
path.write_text(
    text.replace(
        "cutover_rollback_schema_version=kamn.cutover.rollback-evidence.v1",
        "cutover_rollback_schema_version=<drifted>",
        1,
    ),
    encoding="utf-8",
)
PY

set +e
docs_drift_output="$(check_cutover_rollback_docs_contract "$docs_drift_file" 2>&1)"
docs_drift_code=$?
set -e
if [ "$docs_drift_code" -eq 0 ]; then
  echo "expected cutover rollback docs-contract checker to fail closed on docs marker drift" >&2
  exit 1
fi
if ! printf '%s\n' "$docs_drift_output" | grep -q 'cutover_rollback_docs_missing_marker:cutover_rollback_schema_version=kamn.cutover.rollback-evidence.v1'; then
  echo "expected deterministic cutover rollback docs marker drift reason" >&2
  exit 1
fi

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO rollback bundle policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected rollback policy check to keep GO decision"

no_go_bundle="$TMP_DIR/rollback-no-go.json"
no_go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --cutover-manifest-id "cutover-mainnet-2026-02-09" \
    --rollback-trigger-status TRIGGERED \
    --checkpoint-state FAILED \
    --failed-checkpoint-id "" \
    --rollback-target-hash "state-hash-abc" \
    --post-rollback-hash "state-hash-def" \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_generate_output" "final_decision")" "NO-GO" "expected invalid rollback evidence to force NO-GO"

python3 - "$no_go_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
rollback = payload.get("rollback")
if not isinstance(rollback, dict):
    raise SystemExit("expected rollback object in NO-GO cutover rollback evidence bundle")
if rollback.get("trigger_status") != "TRIGGERED":
    raise SystemExit("expected rollback.trigger_status=TRIGGERED for NO-GO path")
if rollback.get("checkpoint_state") != "FAILED":
    raise SystemExit("expected rollback.checkpoint_state=FAILED for NO-GO path")
reason_codes = payload.get("decision_reasons")
if not isinstance(reason_codes, list):
    raise SystemExit("expected decision_reasons list for NO-GO path")
if "missing failed checkpoint evidence" not in reason_codes:
    raise SystemExit("expected missing failed checkpoint reason in NO-GO path")
if "rollback target hash mismatch" not in reason_codes:
    raise SystemExit("expected rollback target hash mismatch reason in NO-GO path")
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected final_decision=NO-GO for NO-GO path")
PY

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO rollback bundle policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected rollback policy check to keep NO-GO decision"

tampered_bundle="$TMP_DIR/rollback-tampered.json"
cp "$no_go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["final_decision"] = "GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered rollback decision bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final-decision mismatch error from rollback policy checker" >&2
  exit 1
fi

# Regression: #708
if ! printf '%s\n' "$tampered_output" | grep -q "missing failed checkpoint evidence"; then
  echo "expected missing failed checkpoint regression guard to be enforced" >&2
  exit 1
fi

echo "cutover rollback evidence bundle tests passed."
