#!/usr/bin/env bash
set -euo pipefail

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-live-deployment-preflight-summary.json"
RUNTIME_MODE="kolme-live"
SIGNER_PROFILE=""
MAX_SECONDS=12
REQUIRED_APPROVALS=2
RECEIVED_APPROVALS=0
CUSTODY_EVIDENCE_FILE=""

REQUIRED_RUNTIME_MODE="kolme-live"
SIGNER_PROFILE_SELECTOR_ENV="KAMN_KOLME_LIVE_SIGNER_PROFILE"
PRIMARY_SIGNER_PROFILE="ops-primary"
SECONDARY_SIGNER_PROFILE="ops-secondary"
PRIMARY_SIGNER_SECRET_ENV="KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
SECONDARY_SIGNER_SECRET_ENV="KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
FALLBACK_SIGNER_SECRET_ENV="KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"
REQUIRED_SECRET_HEX_LENGTH=64

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
    --runtime-mode)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-mode" >&2
        exit 1
      fi
      RUNTIME_MODE="$2"
      shift 2
      ;;
    --signer-profile)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --signer-profile" >&2
        exit 1
      fi
      SIGNER_PROFILE="$2"
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
    --required-approvals)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --required-approvals" >&2
        exit 1
      fi
      REQUIRED_APPROVALS="$2"
      shift 2
      ;;
    --received-approvals)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --received-approvals" >&2
        exit 1
      fi
      RECEIVED_APPROVALS="$2"
      shift 2
      ;;
    --custody-evidence-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --custody-evidence-file" >&2
        exit 1
      fi
      CUSTODY_EVIDENCE_FILE="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_live_deployment_preflight_lane.sh [options]

Options:
  --mode dry-run|run                  Emit planned checks or execute deployment preflight checks.
  --output-json <path>                Deterministic summary report output path.
  --runtime-mode <value>              Runtime mode contract value (must be kolme-live).
  --signer-profile <value>            Signer profile override (ops-primary|ops-secondary).
  --max-seconds <n>                   Max total runtime budget for run mode.
  --required-approvals <n>            Required signer approvals threshold (run-mode fail-closed).
  --received-approvals <n>            Received signer approvals count (run-mode fail-closed).
  --custody-evidence-file <path>      Required signer custody evidence file in run mode.
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

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$MAX_SECONDS" -le 0 ]; then
  echo "max-seconds must be a positive integer" >&2
  exit 1
fi

if ! [[ "$REQUIRED_APPROVALS" =~ ^[0-9]+$ ]] || [ "$REQUIRED_APPROVALS" -le 0 ]; then
  echo "required-approvals must be a positive integer" >&2
  exit 1
fi

if ! [[ "$RECEIVED_APPROVALS" =~ ^[0-9]+$ ]]; then
  echo "received-approvals must be a non-negative integer" >&2
  exit 1
fi

if [ -z "$SIGNER_PROFILE" ]; then
  SIGNER_PROFILE="${KAMN_KOLME_LIVE_SIGNER_PROFILE:-$PRIMARY_SIGNER_PROFILE}"
fi

if [ -z "$SIGNER_PROFILE" ]; then
  echo "signer-profile must not be empty" >&2
  exit 1
fi

selected_signer_secret_env=""
if [ "$SIGNER_PROFILE" = "$PRIMARY_SIGNER_PROFILE" ]; then
  selected_signer_secret_env="$PRIMARY_SIGNER_SECRET_ENV"
elif [ "$SIGNER_PROFILE" = "$SECONDARY_SIGNER_PROFILE" ]; then
  selected_signer_secret_env="$SECONDARY_SIGNER_SECRET_ENV"
fi

CHECK_FILE="$(mktemp)"
trap 'rm -f "$CHECK_FILE"' EXIT

record_check() {
  local check_id="$1"
  local command="$2"
  local status="$3"
  local reason_code="$4"
  printf '%s\t%s\t%s\t%s\n' "$check_id" "$command" "$status" "$reason_code" >>"$CHECK_FILE"
}

runtime_mode_command="runtime-mode must equal ${REQUIRED_RUNTIME_MODE}"
signer_profile_command="signer profile must be ${PRIMARY_SIGNER_PROFILE} or ${SECONDARY_SIGNER_PROFILE}"
signer_secret_command="selected signer secret env must exist and be ${REQUIRED_SECRET_HEX_LENGTH}-char hex"
fallback_private_key_command="fallback signer secret env must remain unset"
signer_quorum_command="received approvals must satisfy required approvals threshold"
custody_evidence_command="signer custody evidence file and sha256 marker must be present"

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
signer_secret_present="false"
signer_secret_hex_valid="false"
fallback_signer_secret_present="false"
custody_evidence_present="false"
custody_evidence_sha256=""
custody_evidence_sha256_valid="false"

record_check "runtime_mode_contract" "$runtime_mode_command" "planned" "not_run"
record_check "signer_profile_contract" "$signer_profile_command" "planned" "not_run"
record_check "signer_secret_contract" "$signer_secret_command" "planned" "not_run"
record_check "fallback_private_key_contract" "$fallback_private_key_command" "planned" "not_run"
record_check "signer_quorum_contract" "$signer_quorum_command" "planned" "not_run"
record_check "custody_evidence_contract" "$custody_evidence_command" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if [ "$RUNTIME_MODE" != "$REQUIRED_RUNTIME_MODE" ]; then
    record_check "runtime_mode_contract" "$runtime_mode_command" "fail" "runtime_mode_mismatch"
    record_check "signer_profile_contract" "$signer_profile_command" "skipped" "runtime_mode_mismatch"
    record_check "signer_secret_contract" "$signer_secret_command" "skipped" "runtime_mode_mismatch"
    record_check "fallback_private_key_contract" "$fallback_private_key_command" "skipped" "runtime_mode_mismatch"
    record_check "signer_quorum_contract" "$signer_quorum_command" "skipped" "runtime_mode_mismatch"
    record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "runtime_mode_mismatch"
    overall_status="fail"
    reason_code="checkpoint_failed_runtime_mode_contract"
  else
    record_check "runtime_mode_contract" "$runtime_mode_command" "pass" "runtime_mode_validated"

    if [ -z "$selected_signer_secret_env" ]; then
      echo "signer profile is invalid for deployment preflight: $SIGNER_PROFILE" >&2
      record_check "signer_profile_contract" "$signer_profile_command" "fail" "signer_profile_invalid"
      record_check "signer_secret_contract" "$signer_secret_command" "skipped" "signer_profile_invalid"
      record_check "fallback_private_key_contract" "$fallback_private_key_command" "skipped" "signer_profile_invalid"
      record_check "signer_quorum_contract" "$signer_quorum_command" "skipped" "signer_profile_invalid"
      record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "signer_profile_invalid"
      overall_status="fail"
      reason_code="checkpoint_failed_signer_profile_contract"
    else
      record_check "signer_profile_contract" "$signer_profile_command" "pass" "signer_profile_validated"

      fallback_signer_secret_value="${!FALLBACK_SIGNER_SECRET_ENV:-}"
      if [ -n "$fallback_signer_secret_value" ]; then
        fallback_signer_secret_present="true"
      fi

      if [ "$fallback_signer_secret_present" = "true" ]; then
        echo "fallback signer secret env must not be set: $FALLBACK_SIGNER_SECRET_ENV" >&2
        record_check "fallback_private_key_contract" "$fallback_private_key_command" "fail" "fallback_signer_secret_present_violation"
        record_check "signer_secret_contract" "$signer_secret_command" "skipped" "fallback_signer_secret_present_violation"
        record_check "signer_quorum_contract" "$signer_quorum_command" "skipped" "fallback_signer_secret_present_violation"
        record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "fallback_signer_secret_present_violation"
        overall_status="fail"
        reason_code="checkpoint_failed_fallback_private_key_contract"
      else
        record_check "fallback_private_key_contract" "$fallback_private_key_command" "pass" "fallback_signer_secret_absent"

      signer_secret_value="${!selected_signer_secret_env:-}"
      if [ -n "$signer_secret_value" ]; then
        signer_secret_present="true"
      fi
      if [[ "$signer_secret_value" =~ ^[0-9a-fA-F]{64}$ ]]; then
        signer_secret_hex_valid="true"
      fi

      if [ "$signer_secret_present" != "true" ]; then
        echo "signer secret env is required for selected profile: $selected_signer_secret_env" >&2
        record_check "signer_secret_contract" "$signer_secret_command" "fail" "signer_secret_missing"
        record_check "signer_quorum_contract" "$signer_quorum_command" "skipped" "signer_secret_missing"
        record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "signer_secret_missing"
        overall_status="fail"
        reason_code="checkpoint_failed_signer_secret_contract"
      elif [ "$signer_secret_hex_valid" != "true" ]; then
        echo "signer secret env must be ${REQUIRED_SECRET_HEX_LENGTH} hex characters: $selected_signer_secret_env" >&2
        record_check "signer_secret_contract" "$signer_secret_command" "fail" "signer_secret_invalid_hex"
        record_check "signer_quorum_contract" "$signer_quorum_command" "skipped" "signer_secret_invalid_hex"
        record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "signer_secret_invalid_hex"
        overall_status="fail"
        reason_code="checkpoint_failed_signer_secret_contract"
      else
        record_check "signer_secret_contract" "$signer_secret_command" "pass" "signer_secret_validated"
        if [ "$RECEIVED_APPROVALS" -lt "$REQUIRED_APPROVALS" ]; then
          echo "signer quorum approvals below required threshold: required=$REQUIRED_APPROVALS received=$RECEIVED_APPROVALS" >&2
          record_check "signer_quorum_contract" "$signer_quorum_command" "fail" "signer_quorum_shortfall"
          record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "signer_quorum_shortfall"
          overall_status="fail"
          reason_code="checkpoint_failed_signer_quorum_contract"
        else
          record_check "signer_quorum_contract" "$signer_quorum_command" "pass" "signer_quorum_validated"
          if [ -n "$CUSTODY_EVIDENCE_FILE" ] && [ -f "$CUSTODY_EVIDENCE_FILE" ]; then
            custody_evidence_present="true"
            custody_evidence_sha256="$(sha256sum "$CUSTODY_EVIDENCE_FILE" | awk '{print $1}')"
            if [[ "$custody_evidence_sha256" =~ ^[0-9a-fA-F]{64}$ ]]; then
              custody_evidence_sha256_valid="true"
            fi
          fi

          if [ "$custody_evidence_present" != "true" ]; then
            echo "signer custody evidence file is required for selected profile" >&2
            record_check "custody_evidence_contract" "$custody_evidence_command" "fail" "custody_evidence_missing"
            overall_status="fail"
            reason_code="checkpoint_failed_custody_evidence_contract"
          elif [ "$custody_evidence_sha256_valid" != "true" ]; then
            echo "signer custody evidence sha256 marker is invalid: $CUSTODY_EVIDENCE_FILE" >&2
            record_check "custody_evidence_contract" "$custody_evidence_command" "fail" "custody_evidence_sha256_invalid"
            overall_status="fail"
            reason_code="checkpoint_failed_custody_evidence_contract"
          else
            record_check "custody_evidence_contract" "$custody_evidence_command" "pass" "custody_evidence_validated"
            reason_code="deployment_preflight_passed"
          fi
        fi
      fi
      fi
    fi
  fi

  elapsed_seconds="$(( $(date +%s) - start_epoch ))"
  if [ "$elapsed_seconds" -le "$MAX_SECONDS" ]; then
    budget_status="within_budget"
  else
    budget_status="exceeded_budget"
    if [ "$overall_status" = "ok" ]; then
      overall_status="fail"
      reason_code="preflight_budget_exceeded"
    fi
  fi
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$RUNTIME_MODE" "$SIGNER_PROFILE_SELECTOR_ENV" "$SIGNER_PROFILE" "$selected_signer_secret_env" "$FALLBACK_SIGNER_SECRET_ENV" "$signer_secret_present" "$fallback_signer_secret_present" "$signer_secret_hex_valid" "$CHECK_FILE" "$REQUIRED_RUNTIME_MODE" "$PRIMARY_SIGNER_PROFILE" "$SECONDARY_SIGNER_PROFILE" "$PRIMARY_SIGNER_SECRET_ENV" "$SECONDARY_SIGNER_SECRET_ENV" "$REQUIRED_SECRET_HEX_LENGTH" "$REQUIRED_APPROVALS" "$RECEIVED_APPROVALS" "$CUSTODY_EVIDENCE_FILE" "$custody_evidence_present" "$custody_evidence_sha256" "$custody_evidence_sha256_valid" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
elapsed_seconds = int(sys.argv[5])
max_seconds = int(sys.argv[6])
budget_status = sys.argv[7]
runtime_mode = sys.argv[8]
signer_profile_selector_env = sys.argv[9]
signer_profile = sys.argv[10]
signer_private_key_env = sys.argv[11]
fallback_signer_private_key_env = sys.argv[12]
signer_secret_present = sys.argv[13] == "true"
fallback_signer_secret_present = sys.argv[14] == "true"
signer_secret_hex_valid = sys.argv[15] == "true"
checks_path = pathlib.Path(sys.argv[16])
required_runtime_mode = sys.argv[17]
primary_signer_profile = sys.argv[18]
secondary_signer_profile = sys.argv[19]
primary_signer_secret_env = sys.argv[20]
secondary_signer_secret_env = sys.argv[21]
required_secret_hex_length = int(sys.argv[22])
required_approvals = int(sys.argv[23])
received_approvals = int(sys.argv[24])
custody_evidence_file = sys.argv[25]
custody_evidence_present = sys.argv[26] == "true"
custody_evidence_sha256 = sys.argv[27]
custody_evidence_sha256_valid = sys.argv[28] == "true"

checks = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 4:
        continue
    check_id, command, check_status, check_reason = parts
    checks.append(
        {
            "id": check_id,
            "command": command,
            "status": check_status,
            "reason_code": check_reason,
        }
    )

summary = {
    "schema_version": "kamn.kolme.local-live-deployment-preflight-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": False,
    "ci_fast_gate_eligible": True,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "runtime_mode": runtime_mode,
    "signer_profile_selector_env": signer_profile_selector_env,
    "signer_profile": signer_profile,
    "signer_private_key_env": signer_private_key_env,
    "fallback_signer_private_key_env": fallback_signer_private_key_env,
    "signer_secret_present": signer_secret_present,
    "fallback_signer_secret_present": fallback_signer_secret_present,
    "signer_secret_hex_valid": signer_secret_hex_valid,
    "required_approvals": required_approvals,
    "received_approvals": received_approvals,
    "custody_evidence_file": custody_evidence_file,
    "custody_evidence_present": custody_evidence_present,
    "custody_evidence_sha256": custody_evidence_sha256,
    "custody_evidence_sha256_valid": custody_evidence_sha256_valid,
    "contracts": {
        "ci_fast_gate_scope": "ci-fast-gate",
        "required_runtime_mode": required_runtime_mode,
        "signer_profile_selector_env": signer_profile_selector_env,
        "supported_signer_profiles": [primary_signer_profile, secondary_signer_profile],
        "primary_signer_secret_env": primary_signer_secret_env,
        "secondary_signer_secret_env": secondary_signer_secret_env,
        "fallback_signer_secret_env": fallback_signer_private_key_env,
        "fallback_private_key_path_allowed": False,
        "required_secret_hex_length": required_secret_hex_length,
        "secret_source": "env",
        "approval_quorum_required": required_approvals,
        "approval_quorum_source": "local-operator-attestations",
        "custody_evidence_required": True,
        "custody_evidence_sha256_required": True,
    },
    "checks": checks,
    "artifact_paths": [],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "ci_fast_gate_eligible=true"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
