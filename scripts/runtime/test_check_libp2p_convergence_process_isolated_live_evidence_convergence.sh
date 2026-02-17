#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_libp2p_convergence_process_isolated_live_contract_lane.sh"
EVIDENCE_CHECKER="$ROOT_DIR/scripts/runtime/check_libp2p_convergence_process_isolated_live_evidence_convergence.sh"

for required_exec in "$CONTRACT_LANE" "$EVIDENCE_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected process-isolated convergence script to be executable: $required_exec" >&2
    exit 1
  fi
done

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
lane_report="$TMP_DIR/libp2p-convergence-process-isolated-contract-lane-report.json"
policy_report="$TMP_DIR/libp2p-convergence-process-isolated-policy-report.json"
summary_report="$TMP_DIR/libp2p-convergence-process-isolated-summary-report.json"
convergence_report="$TMP_DIR/libp2p-convergence-process-isolated-convergence-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --lane-profile smoke \
    --ci-fast-gate PASS \
    --max-seconds 180 \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report" \
    --summary-output-json "$summary_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected process-isolated convergence contract lane status marker" >&2
  exit 1
fi

convergence_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$policy_report" \
    --output-json "$convergence_report"
)"
if ! printf '%s\n' "$convergence_output" | grep -q '^status=ok$'; then
  echo "expected process-isolated convergence evidence checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^final_decision=GO$'; then
  echo "expected process-isolated convergence evidence checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^evidence_convergence_status=verified$'; then
  echo "expected deterministic evidence convergence status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^promotion_decision_reason_mapping_status=verified$'; then
  echo "expected deterministic promotion decision reason mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-evidence-convergence-reason-taxonomy.v1$'; then
  echo "expected deterministic evidence reason taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^reason_codes_csv=libp2p_finality_evidence_link_missing,libp2p_finality_evidence_payload_tamper_detected,libp2p_finality_promotion_decision_reason_mapping_mismatch$'; then
  echo "expected deterministic evidence reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected deterministic evidence reason codes value marker on GO path" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^promotion_decision_reason_taxonomy_version=kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1$'; then
  echo "expected deterministic promotion decision reason taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^promotion_decision_reason_codes_csv=libp2p_process_isolated_convergence_policy_required_field_missing,libp2p_process_isolated_convergence_policy_marker_missing,libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch,libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch,finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,libp2p_process_isolated_convergence_policy_expected_decision_mismatch,libp2p_process_isolated_convergence_policy_violation$'; then
  echo "expected deterministic promotion decision reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^promotion_decision_reason_code=none$'; then
  echo "expected deterministic promotion decision reason code marker on GO path" >&2
  exit 1
fi

python3 - "$convergence_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.libp2p-convergence-process-isolated-live-convergence-report.v1":
    raise SystemExit("unexpected process-isolated convergence evidence report schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected process-isolated convergence evidence final_decision=GO")
if payload.get("evidence_convergence_status") != "verified":
    raise SystemExit("expected deterministic evidence_convergence_status marker")
if payload.get("promotion_decision_reason_mapping_status") != "verified":
    raise SystemExit("expected deterministic promotion_decision_reason_mapping_status marker")
if payload.get("reason_taxonomy_version") != "kamn.runtime.libp2p-fork-choice-finality-evidence-convergence-reason-taxonomy.v1":
    raise SystemExit("expected deterministic evidence reason taxonomy version marker")
if payload.get("reason_codes_csv") != "libp2p_finality_evidence_link_missing,libp2p_finality_evidence_payload_tamper_detected,libp2p_finality_promotion_decision_reason_mapping_mismatch":
    raise SystemExit("expected deterministic evidence reason taxonomy csv marker")
if payload.get("reason_codes_value") != "none":
    raise SystemExit("expected deterministic evidence reason_codes_value=none marker")
if payload.get("promotion_decision_reason_taxonomy_version") != "kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1":
    raise SystemExit("expected deterministic promotion decision reason taxonomy version marker")
if payload.get("promotion_decision_reason_codes_csv") != "libp2p_process_isolated_convergence_policy_required_field_missing,libp2p_process_isolated_convergence_policy_marker_missing,libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch,libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch,finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,libp2p_process_isolated_convergence_policy_expected_decision_mismatch,libp2p_process_isolated_convergence_policy_violation":
    raise SystemExit("expected deterministic promotion decision reason taxonomy csv marker")
if payload.get("promotion_decision_reason_code") != "none":
    raise SystemExit("expected deterministic promotion decision reason code marker on GO path")
PY

tampered_policy_report="$TMP_DIR/libp2p-convergence-process-isolated-policy-report.tampered-mapping.json"
cp "$policy_report" "$tampered_policy_report"
python3 - "$tampered_policy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["promotion_decision_reason_code"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_mapping_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$tampered_policy_report" \
    --output-json "$TMP_DIR/libp2p-convergence-process-isolated-convergence-report.tampered-mapping.json" 2>&1
)"
tampered_mapping_code=$?
set -e
if [ "$tampered_mapping_code" -eq 0 ]; then
  echo "expected tampered promotion decision reason mapping to fail evidence convergence checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_mapping_output" | grep -q 'libp2p_finality_promotion_decision_reason_mapping_mismatch'; then
  echo "expected deterministic promotion decision reason mapping mismatch marker" >&2
  exit 1
fi

missing_link_policy_report="$TMP_DIR/libp2p-convergence-process-isolated-policy-report.missing-link.json"
cp "$policy_report" "$missing_link_policy_report"
python3 - "$missing_link_policy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["source_report_file"] = ""
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
missing_link_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$missing_link_policy_report" \
    --output-json "$TMP_DIR/libp2p-convergence-process-isolated-convergence-report.missing-link.json" 2>&1
)"
missing_link_code=$?
set -e
if [ "$missing_link_code" -eq 0 ]; then
  echo "expected missing source report link to fail evidence convergence checker" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_link_output" | grep -q 'libp2p_finality_evidence_link_missing:source_report_file'; then
  echo "expected deterministic missing evidence link marker for source_report_file" >&2
  exit 1
fi

echo "process-isolated libp2p convergence evidence checker tests passed."
