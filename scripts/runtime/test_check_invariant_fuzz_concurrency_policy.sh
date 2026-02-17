#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/runtime/check_invariant_fuzz_concurrency_policy.sh"
LANE_SCRIPT="$ROOT_DIR/scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh"
EXPECTED_REASON_TAXONOMY_VERSION="kamn.runtime.invariant-fuzz-concurrency-policy-reason-taxonomy.v1"
EXPECTED_REASON_CODES_CSV="property_lane_failed,fuzz_lane_failed,concurrency_lane_failed,runtime_budget_exceeded,ci_smoke_local_heavy_boundary_status_mismatch,ci_smoke_lane_cost_profile_mismatch,local_heavy_lane_execution_mode_mismatch,missing_required_report_fields,schema_version_mismatch,status_value_invalid,lane_status_value_invalid,property_replay_schema_version_mismatch,property_replay_artifact_key_mismatch,property_replay_test_count_invalid,fuzz_replay_schema_version_mismatch,fuzz_replay_artifact_key_mismatch,fuzz_replay_test_count_invalid,concurrency_replay_schema_version_mismatch,concurrency_replay_artifact_key_mismatch,concurrency_replay_test_count_invalid,elapsed_seconds_invalid,max_seconds_invalid,reason_codes_payload_invalid,status_contract_mismatch,reason_codes_contract_mismatch,reason_taxonomy_version_mismatch,reason_codes_csv_mismatch,reason_codes_value_mismatch,final_decision_mismatch"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected invariant/fuzz/concurrency policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected invariant/fuzz/concurrency contract lane script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/invariant-fuzz-concurrency-contract-report.json"
bash "$LANE_SCRIPT" --output-json "$report_file" >/dev/null

go_output="$(bash "$CHECKER" --report-file "$report_file")"
if ! printf '%s\n' "$go_output" | grep -Fq "status=ok"; then
  echo "expected invariant/fuzz/concurrency policy checker success status" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -Fq "invariant_policy_status=verified"; then
  echo "expected invariant/fuzz/concurrency policy checker verified status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -Fq "invariant_policy_reason_taxonomy_version=$EXPECTED_REASON_TAXONOMY_VERSION"; then
  echo "expected deterministic invariant/fuzz/concurrency reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -Fq "invariant_policy_reason_codes_csv=$EXPECTED_REASON_CODES_CSV"; then
  echo "expected deterministic invariant/fuzz/concurrency reason codes csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -Fq "invariant_policy_reason_codes_value=none"; then
  echo "expected invariant/fuzz/concurrency checker pass path reason value marker" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -Fq "invariant_policy_expected_reason_codes_value=none"; then
  echo "expected invariant/fuzz/concurrency checker expected reason mapping marker" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -Fq "invariant_policy_observed_reason_codes_value=none"; then
  echo "expected invariant/fuzz/concurrency checker observed reason mapping marker" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -Fq "invariant_policy_ci_smoke_local_heavy_boundary_status=verified"; then
  echo "expected invariant/fuzz/concurrency checker boundary status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -Fq "invariant_policy_ci_smoke_lane_cost_profile=low"; then
  echo "expected invariant/fuzz/concurrency checker ci smoke cost-profile marker" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -Fq "invariant_policy_local_heavy_lane_execution_mode=opt_in"; then
  echo "expected invariant/fuzz/concurrency checker local-heavy execution mode marker" >&2
  exit 1
fi

tampered_report="$TMP_DIR/invariant-fuzz-concurrency-contract-report.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["concurrency_replay_artifact_key"] = "tampered_concurrency_artifact_key"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered invariant/fuzz/concurrency report to fail policy checker" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -Fq "invariant_policy_reason_codes_value=concurrency_replay_artifact_key_mismatch"; then
  echo "expected deterministic concurrency replay artifact mismatch reason marker" >&2
  exit 1
fi

# Regression: #1363
if ! printf '%s\n' "$tampered_output" | grep -Fq "invariant_policy_final_decision=NO-GO"; then
  echo "expected fail-closed final decision marker in policy regression path" >&2
  exit 1
fi

tampered_lane_violation_report="$TMP_DIR/invariant-fuzz-concurrency-contract-report.lane-violation.json"
cp "$report_file" "$tampered_lane_violation_report"
python3 - "$tampered_lane_violation_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["property_lane_status"] = "fail"
payload["status"] = "pass"
payload["reason_codes"] = ["none"]
payload["reason_codes_value"] = "none"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_lane_violation_output="$(bash "$CHECKER" --report-file "$tampered_lane_violation_report" 2>&1)"
tampered_lane_violation_code=$?
set -e

if [ "$tampered_lane_violation_code" -eq 0 ]; then
  echo "expected lane-violation acceptance drift report to fail policy checker" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_lane_violation_output" | grep -Fq "invariant_policy_expected_reason_codes_value=property_lane_failed"; then
  echo "expected deterministic expected reason mapping marker for lane failure acceptance drift" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_lane_violation_output" | grep -Fq "invariant_policy_reason_codes_value=status_contract_mismatch,reason_codes_contract_mismatch"; then
  echo "expected deterministic mismatch reason codes for lane failure acceptance drift" >&2
  exit 1
fi

tampered_fuzz_count_report="$TMP_DIR/invariant-fuzz-concurrency-contract-report.fuzz-count.json"
cp "$report_file" "$tampered_fuzz_count_report"
python3 - "$tampered_fuzz_count_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["fuzz_replay_test_count"] = 0
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_fuzz_count_output="$(bash "$CHECKER" --report-file "$tampered_fuzz_count_report" 2>&1)"
tampered_fuzz_count_code=$?
set -e

if [ "$tampered_fuzz_count_code" -eq 0 ]; then
  echo "expected fuzz replay-count regression drift report to fail policy checker" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_fuzz_count_output" | grep -Fq "invariant_policy_reason_codes_value=fuzz_replay_test_count_invalid"; then
  echo "expected deterministic fuzz replay-count mismatch reason marker" >&2
  exit 1
fi

tampered_concurrency_misclassification_report="$TMP_DIR/invariant-fuzz-concurrency-contract-report.concurrency-misclassification.json"
cp "$report_file" "$tampered_concurrency_misclassification_report"
python3 - "$tampered_concurrency_misclassification_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["concurrency_lane_status"] = "fail"
payload["status"] = "pass"
payload["reason_codes"] = ["fuzz_lane_failed"]
payload["reason_codes_value"] = "fuzz_lane_failed"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_concurrency_misclassification_output="$(bash "$CHECKER" --report-file "$tampered_concurrency_misclassification_report" 2>&1)"
tampered_concurrency_misclassification_code=$?
set -e

if [ "$tampered_concurrency_misclassification_code" -eq 0 ]; then
  echo "expected concurrency race misclassification report to fail policy checker" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_concurrency_misclassification_output" | grep -Fq "invariant_policy_expected_reason_codes_value=concurrency_lane_failed"; then
  echo "expected deterministic expected reason mapping marker for concurrency misclassification drift" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_concurrency_misclassification_output" | grep -Fq "invariant_policy_reason_codes_value=status_contract_mismatch,reason_codes_contract_mismatch"; then
  echo "expected deterministic mismatch reason markers for concurrency race misclassification drift" >&2
  exit 1
fi

tampered_boundary_report="$TMP_DIR/invariant-fuzz-concurrency-contract-report.boundary-tampered.json"
cp "$report_file" "$tampered_boundary_report"
python3 - "$tampered_boundary_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["ci_smoke_local_heavy_boundary_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_boundary_output="$(bash "$CHECKER" --report-file "$tampered_boundary_report" 2>&1)"
tampered_boundary_code=$?
set -e

if [ "$tampered_boundary_code" -eq 0 ]; then
  echo "expected CI/local-heavy boundary drift report to fail policy checker" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_boundary_output" | grep -Fq "invariant_policy_reason_codes_value=ci_smoke_local_heavy_boundary_status_mismatch"; then
  echo "expected deterministic CI/local-heavy boundary mismatch reason marker" >&2
  exit 1
fi

tampered_taxonomy_report="$TMP_DIR/invariant-fuzz-concurrency-contract-report.taxonomy-tampered.json"
cp "$report_file" "$tampered_taxonomy_report"
python3 - "$tampered_taxonomy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_taxonomy_version"] = "kamn.runtime.invariant-fuzz-concurrency-policy-reason-taxonomy.v0"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_taxonomy_output="$(bash "$CHECKER" --report-file "$tampered_taxonomy_report" 2>&1)"
tampered_taxonomy_code=$?
set -e

if [ "$tampered_taxonomy_code" -eq 0 ]; then
  echo "expected taxonomy-tampered report to fail policy checker" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_taxonomy_output" | grep -Fq "invariant_policy_reason_codes_value=reason_taxonomy_version_mismatch"; then
  echo "expected deterministic taxonomy mismatch reason marker" >&2
  exit 1
fi

echo "invariant/fuzz/concurrency evidence policy checker tests passed."
