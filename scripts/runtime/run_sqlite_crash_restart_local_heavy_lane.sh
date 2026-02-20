#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SOURCE_LANE="$ROOT_DIR/scripts/runtime/validate_sqlite_crash_recovery_live_contract_lane.sh"

REPORT_SCHEMA_VERSION="kamn.runtime.sqlite-crash-restart-local-heavy-lane-report.v1"
ARTIFACT_SCHEMA_VERSION="kamn.runtime.sqlite-crash-restart-local-heavy-artifact-schema.v1"
REASON_TAXONOMY_VERSION="kamn.runtime.sqlite-crash-restart-local-heavy-reason-taxonomy.v1"
REASON_CODES_CSV="crash_restart_profile_restart_status_mismatch,crash_restart_profile_corruption_status_mismatch,crash_restart_profile_combined_status_mismatch"

profile="combined"
mode="dry-run"
ci_fast_gate="PASS"
max_seconds="${KAMN_SQLITE_CRASH_RESTART_LOCAL_HEAVY_MAX_SECONDS:-240}"
output_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      profile="${2:-}"
      shift 2
      ;;
    --mode)
      mode="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ "$profile" != "restart" && "$profile" != "corruption" && "$profile" != "combined" ]]; then
  echo "profile must be restart, corruption, or combined" >&2
  exit 1
fi
if [[ "$mode" != "dry-run" && "$mode" != "run" ]]; then
  echo "mode must be dry-run or run" >&2
  exit 1
fi
if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  echo "ci-fast-gate must be PASS or FAIL" >&2
  exit 1
fi
if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi
if [ ! -x "$SOURCE_LANE" ]; then
  echo "expected required executable script '$SOURCE_LANE'" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

source_report="$TMP_DIR/sqlite-crash-restart-source-report.json"
source_policy="$TMP_DIR/sqlite-crash-restart-source-policy.json"
source_summary="$TMP_DIR/sqlite-crash-restart-source-summary.json"
source_convergence="$TMP_DIR/sqlite-crash-restart-source-convergence.json"

bash "$SOURCE_LANE" \
  --mode "$mode" \
  --max-seconds "$max_seconds" \
  --ci-fast-gate "$ci_fast_gate" \
  --output-json "$source_report" \
  --policy-output-json "$source_policy" \
  --summary-output-json "$source_summary" \
  --convergence-output-json "$source_convergence" >/dev/null

python3 - "$source_report" "$output_json" "$profile" "$mode" "$ci_fast_gate" \
  "$REPORT_SCHEMA_VERSION" "$ARTIFACT_SCHEMA_VERSION" "$REASON_TAXONOMY_VERSION" "$REASON_CODES_CSV" <<'PY'
import json
import pathlib
import sys

(
    source_report_path,
    output_json_path,
    profile,
    mode,
    ci_fast_gate,
    report_schema,
    artifact_schema,
    reason_taxonomy,
    reason_codes_csv,
) = sys.argv[1:]

source_report = json.loads(pathlib.Path(source_report_path).read_text(encoding="utf-8"))

source_status = source_report.get("status")
source_final_decision = source_report.get("final_decision")
source_wal_append_status = source_report.get("wal_append_status")
source_wal_checkpoint_status = source_report.get("wal_checkpoint_status")
source_append_checkpoint_integrity_status = source_report.get("append_checkpoint_integrity_status")
source_journal_replay_status = source_report.get("journal_replay_drift_detection_status")
source_checkpoint_bypass_status = source_report.get("checkpoint_divergence_bypass_rejection_status")
source_readiness_progress_status = source_report.get("crash_recovery_readiness_progress_status")
source_snapshot_parity_status = source_report.get("snapshot_parity_status")

restart_markers_verified = all(
    value == "verified"
    for value in [
        source_journal_replay_status,
        source_checkpoint_bypass_status,
        source_readiness_progress_status,
        source_snapshot_parity_status,
    ]
)

corruption_markers_verified = all(
    value == "verified"
    for value in [
        source_wal_append_status,
        source_wal_checkpoint_status,
        source_append_checkpoint_integrity_status,
    ]
)

if profile == "restart":
    restart_drill_status = "verified" if restart_markers_verified else "failed"
    corruption_drill_status = "not_applicable"
    profile_status = "verified" if restart_drill_status == "verified" else "failed"
    reason_code = "none" if profile_status == "verified" else "crash_restart_profile_restart_status_mismatch"
elif profile == "corruption":
    restart_drill_status = "not_applicable"
    corruption_drill_status = "verified" if corruption_markers_verified else "failed"
    profile_status = "verified" if corruption_drill_status == "verified" else "failed"
    reason_code = "none" if profile_status == "verified" else "crash_restart_profile_corruption_status_mismatch"
else:
    restart_drill_status = "verified" if restart_markers_verified else "failed"
    corruption_drill_status = "verified" if corruption_markers_verified else "failed"
    profile_status = (
        "verified"
        if restart_drill_status == "verified" and corruption_drill_status == "verified"
        else "failed"
    )
    reason_code = "none" if profile_status == "verified" else "crash_restart_profile_combined_status_mismatch"

status = (
    "pass"
    if profile_status == "verified" and source_status == "pass" and source_final_decision == "GO"
    else "fail"
)
final_decision = "GO" if status == "pass" else "NO-GO"
if final_decision == "GO":
    reason_code = "none"

report = {
    "schema_version": report_schema,
    "artifact_schema_version": artifact_schema,
    "reason_taxonomy_version": reason_taxonomy,
    "reason_codes_csv": reason_codes_csv,
    "status": status,
    "final_decision": final_decision,
    "lane_mode": mode,
    "profile": profile,
    "profile_status": profile_status,
    "reason_code": reason_code,
    "restart_drill_status": restart_drill_status,
    "corruption_drill_status": corruption_drill_status,
    "ci_fast_gate": ci_fast_gate,
    "source_report_schema_version": source_report.get("schema_version", "missing"),
    "source_command_count": source_report.get("command_count", 0),
    "source_policy_status": source_report.get("sqlite_crash_recovery_policy_status", "missing"),
}

if output_json_path:
    output_path = pathlib.Path(output_json_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

print(f"status={report['status']}")
print(f"final_decision={report['final_decision']}")
print(f"lane_mode={report['lane_mode']}")
print(f"profile={report['profile']}")
print(f"profile_status={report['profile_status']}")
print(f"reason_code={report['reason_code']}")
print(f"restart_drill_status={report['restart_drill_status']}")
print(f"corruption_drill_status={report['corruption_drill_status']}")
print(f"schema_version={report['schema_version']}")
print(f"artifact_schema_version={report['artifact_schema_version']}")
print(f"reason_taxonomy_version={report['reason_taxonomy_version']}")
print(f"reason_codes_csv={report['reason_codes_csv']}")
print(f"source_report_schema_version={report['source_report_schema_version']}")
print(f"source_command_count={report['source_command_count']}")
PY
