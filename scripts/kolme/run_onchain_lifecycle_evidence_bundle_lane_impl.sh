#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_TOOL="$ROOT_DIR/scripts/kolme/check_onchain_lifecycle_evidence_policy.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-onchain-lifecycle-evidence-bundle-summary.json"
DID_REPORT_FILE="/tmp/kolme-did-lifecycle-live-validation-report.json"
MESSAGE_REPORT_FILE="/tmp/kolme-message-proof-live-validation-report.json"
RUNTIME_REPORT_FILE="/tmp/kolme-continuous-runtime-live-validation-report.json"
MAX_SECONDS=480
DID_MAX_SECONDS=180
MESSAGE_MAX_SECONDS=180
RUNTIME_MAX_SECONDS=180
DID_COMMAND=""
MESSAGE_COMMAND=""
RUNTIME_COMMAND=""

shell_escape() {
  printf "%q" "$1"
}

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

write_dry_run_fixture_reports() {
  mkdir -p "$(dirname "$DID_REPORT_FILE")"
  mkdir -p "$(dirname "$MESSAGE_REPORT_FILE")"
  mkdir -p "$(dirname "$RUNTIME_REPORT_FILE")"

  cat >"$DID_REPORT_FILE" <<'JSON'
{
  "schema_version": "kamn.kolme.did-lifecycle-chain.live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "did_lifecycle_contract_status": "verified",
  "evidence_bundle_status": "verified",
  "docs_contract_status": "verified",
  "fail_closed_status": "verified",
  "fail_closed_reason_code": "did_registry_submission_key_conflict",
  "performance_budget_status": "verified"
}
JSON

  cat >"$MESSAGE_REPORT_FILE" <<'JSON'
{
  "schema_version": "kamn.kolme.message-proof-anchoring.live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "message_anchor_contract_status": "verified",
  "evidence_bundle_status": "verified",
  "docs_contract_status": "verified",
  "fail_closed_status": "verified",
  "fail_closed_reason_code": "message_proof_anchor_conflicting_key",
  "performance_budget_status": "verified"
}
JSON

  cat >"$RUNTIME_REPORT_FILE" <<'JSON'
{
  "schema_version": "kamn.kolme.continuous-runtime-commit.live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "continuous_runtime_contract_status": "verified",
  "evidence_bundle_status": "verified",
  "fail_closed_status": "verified",
  "fail_closed_reason_code": "paired_cycle_controls_required"
}
JSON
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --mode" >&2
        exit 1
      fi
      MODE="$2"
      shift 2
      ;;
    --output-json)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --output-json" >&2
        exit 1
      fi
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --did-report-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --did-report-file" >&2
        exit 1
      fi
      DID_REPORT_FILE="$2"
      shift 2
      ;;
    --message-report-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --message-report-file" >&2
        exit 1
      fi
      MESSAGE_REPORT_FILE="$2"
      shift 2
      ;;
    --runtime-report-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-report-file" >&2
        exit 1
      fi
      RUNTIME_REPORT_FILE="$2"
      shift 2
      ;;
    --max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --max-seconds" >&2
        exit 1
      fi
      MAX_SECONDS="$2"
      shift 2
      ;;
    --did-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --did-max-seconds" >&2
        exit 1
      fi
      DID_MAX_SECONDS="$2"
      shift 2
      ;;
    --message-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --message-max-seconds" >&2
        exit 1
      fi
      MESSAGE_MAX_SECONDS="$2"
      shift 2
      ;;
    --runtime-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-max-seconds" >&2
        exit 1
      fi
      RUNTIME_MAX_SECONDS="$2"
      shift 2
      ;;
    --did-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --did-command" >&2
        exit 1
      fi
      DID_COMMAND="$2"
      shift 2
      ;;
    --message-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --message-command" >&2
        exit 1
      fi
      MESSAGE_COMMAND="$2"
      shift 2
      ;;
    --runtime-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-command" >&2
        exit 1
      fi
      RUNTIME_COMMAND="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_onchain_lifecycle_evidence_bundle_lane.sh [options]

Options:
  --mode dry-run|run                    Emit planned artifacts or execute live validations.
  --output-json <path>                  Deterministic aggregate summary output.
  --did-report-file <path>              DID lifecycle report output/input path.
  --message-report-file <path>          Message proof report output/input path.
  --runtime-report-file <path>          Runtime commit report output/input path.
  --max-seconds <n>                     Total aggregate runtime budget in seconds.
  --did-max-seconds <n>                 DID validator budget for default run command.
  --message-max-seconds <n>             Message validator budget for default run command.
  --runtime-max-seconds <n>             Runtime validator budget for default run command.
  --did-command <command>               Override DID validator command.
  --message-command <command>           Override message validator command.
  --runtime-command <command>           Override runtime validator command.
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [ "$MODE" != "dry-run" ] && [ "$MODE" != "run" ]; then
  echo "mode must be one of: dry-run, run" >&2
  exit 1
fi

for numeric_value in "$MAX_SECONDS" "$DID_MAX_SECONDS" "$MESSAGE_MAX_SECONDS" "$RUNTIME_MAX_SECONDS"; do
  if ! [[ "$numeric_value" =~ ^[0-9]+$ ]] || [ "$numeric_value" -le 0 ]; then
    echo "all runtime budgets must be positive integers" >&2
    exit 1
  fi
done

if [ ! -x "$POLICY_TOOL" ]; then
  echo "expected on-chain lifecycle policy tool to be executable" >&2
  exit 1
fi

if [ -z "$DID_COMMAND" ]; then
  DID_COMMAND="bash scripts/kolme/validate_did_lifecycle_chain_adapter_live.sh --max-seconds $DID_MAX_SECONDS --output-json $(shell_escape "$DID_REPORT_FILE")"
fi
if [ -z "$MESSAGE_COMMAND" ]; then
  MESSAGE_COMMAND="bash scripts/kolme/validate_message_proof_anchoring_live.sh --max-seconds $MESSAGE_MAX_SECONDS --output-json $(shell_escape "$MESSAGE_REPORT_FILE")"
fi
if [ -z "$RUNTIME_COMMAND" ]; then
  RUNTIME_COMMAND="bash scripts/kolme/validate_continuous_runtime_commit_live.sh --max-seconds $RUNTIME_MAX_SECONDS --output-json $(shell_escape "$RUNTIME_REPORT_FILE")"
fi

elapsed_seconds=0
budget_status="not_run"
reason_code="dry_run_no_commands_executed"
command_failure_reason=""

if [ "$MODE" = "dry-run" ]; then
  write_dry_run_fixture_reports
else
  bash "$LOCAL_HEAVY_GUARD"
  run_start_epoch="$(date +%s)"
  for check_entry in "did:$DID_COMMAND:did_lifecycle_validation_command_failed" \
    "message:$MESSAGE_COMMAND:message_proof_validation_command_failed" \
    "runtime:$RUNTIME_COMMAND:runtime_commit_validation_command_failed"; do
    check_id="${check_entry%%:*}"
    remainder="${check_entry#*:}"
    check_command="${remainder%:*}"
    check_reason="${check_entry##*:}"

    set +e
    command_output="$(
      cd "$ROOT_DIR"
      bash -lc "$check_command"
    2>&1)"
    command_code=$?
    set -e
    if [ "$command_code" -ne 0 ]; then
      printf '%s\n' "$command_output" >&2
      if [ -z "$command_failure_reason" ]; then
        command_failure_reason="$check_reason"
      fi
    fi
    if [ "$check_id" = "did" ] && [ ! -f "$DID_REPORT_FILE" ]; then
      :
    fi
  done

  elapsed_seconds="$(( $(date +%s) - run_start_epoch ))"
  if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
    budget_status="exceeded_budget"
    if [ -z "$command_failure_reason" ]; then
      command_failure_reason="onchain_lifecycle_bundle_runtime_budget_exceeded"
    fi
  else
    budget_status="within_budget"
  fi

  if [ -n "$command_failure_reason" ]; then
    reason_code="$command_failure_reason"
  else
    reason_code="live_onchain_lifecycle_bundle_passed"
  fi
fi

set +e
generate_output="$(
  python3 "$POLICY_TOOL" generate \
    --mode "$MODE" \
    --did-report-file "$DID_REPORT_FILE" \
    --message-report-file "$MESSAGE_REPORT_FILE" \
    --runtime-report-file "$RUNTIME_REPORT_FILE" \
    --max-seconds "$MAX_SECONDS" \
    --elapsed-seconds "$elapsed_seconds" \
    --budget-status "$budget_status" \
    --reason-code "$reason_code" \
    --output-json "$OUTPUT_JSON"
)"
generate_code=$?
set -e

summary_status="$(extract_value "$generate_output" "status")"
final_decision="$(extract_value "$generate_output" "final_decision")"
finality_lineage_status="$(extract_value "$generate_output" "finality_lineage_status")"
recovery_lineage_status="$(extract_value "$generate_output" "recovery_lineage_status")"

echo "status=${summary_status:-fail}"
echo "lane_mode=$MODE"
echo "final_decision=${final_decision:-NO-GO}"
echo "reason_code=$reason_code"
echo "local_only_enforced=true"
echo "ci_fast_gate_eligible=false"
echo "finality_lineage_status=${finality_lineage_status:-missing}"
echo "recovery_lineage_status=${recovery_lineage_status:-missing}"

if [ "$generate_code" -ne 0 ]; then
  exit "$generate_code"
fi

if [ "${final_decision:-NO-GO}" != "GO" ]; then
  exit 1
fi
