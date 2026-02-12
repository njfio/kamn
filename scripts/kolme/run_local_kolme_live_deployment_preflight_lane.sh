#!/usr/bin/env bash
set -euo pipefail

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-live-deployment-preflight-summary.json"
RUNTIME_MODE="kolme-live"
SIGNER_PROFILE=""
MAX_SECONDS=12

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
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_live_deployment_preflight_lane.sh [options]

Options:
  --mode dry-run|run                  Emit planned checks or execute deployment preflight checks.
  --output-json <path>                Deterministic summary report output path.
  --runtime-mode <value>              Runtime mode contract value (must be kolme-live).
  --signer-profile <value>            Signer profile override (ops-primary|ops-secondary).
  --max-seconds <n>                   Max total runtime budget for run mode.
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

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
signer_secret_present="false"
signer_secret_hex_valid="false"
fallback_signer_secret_present="false"

record_check "runtime_mode_contract" "$runtime_mode_command" "planned" "not_run"
record_check "signer_profile_contract" "$signer_profile_command" "planned" "not_run"
record_check "signer_secret_contract" "$signer_secret_command" "planned" "not_run"
record_check "fallback_private_key_contract" "$fallback_private_key_command" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if [ "$RUNTIME_MODE" != "$REQUIRED_RUNTIME_MODE" ]; then
    record_check "runtime_mode_contract" "$runtime_mode_command" "fail" "runtime_mode_mismatch"
    record_check "signer_profile_contract" "$signer_profile_command" "skipped" "runtime_mode_mismatch"
    record_check "signer_secret_contract" "$signer_secret_command" "skipped" "runtime_mode_mismatch"
    record_check "fallback_private_key_contract" "$fallback_private_key_command" "skipped" "runtime_mode_mismatch"
    overall_status="fail"
    reason_code="checkpoint_failed_runtime_mode_contract"
  else
    record_check "runtime_mode_contract" "$runtime_mode_command" "pass" "runtime_mode_validated"

    if [ -z "$selected_signer_secret_env" ]; then
      echo "signer profile is invalid for deployment preflight: $SIGNER_PROFILE" >&2
      record_check "signer_profile_contract" "$signer_profile_command" "fail" "signer_profile_invalid"
      record_check "signer_secret_contract" "$signer_secret_command" "skipped" "signer_profile_invalid"
      record_check "fallback_private_key_contract" "$fallback_private_key_command" "skipped" "signer_profile_invalid"
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
        overall_status="fail"
        reason_code="checkpoint_failed_signer_secret_contract"
      elif [ "$signer_secret_hex_valid" != "true" ]; then
        echo "signer secret env must be ${REQUIRED_SECRET_HEX_LENGTH} hex characters: $selected_signer_secret_env" >&2
        record_check "signer_secret_contract" "$signer_secret_command" "fail" "signer_secret_invalid_hex"
        overall_status="fail"
        reason_code="checkpoint_failed_signer_secret_contract"
      else
        record_check "signer_secret_contract" "$signer_secret_command" "pass" "signer_secret_validated"
        reason_code="deployment_preflight_passed"
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

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$RUNTIME_MODE" "$SIGNER_PROFILE_SELECTOR_ENV" "$SIGNER_PROFILE" "$selected_signer_secret_env" "$FALLBACK_SIGNER_SECRET_ENV" "$signer_secret_present" "$fallback_signer_secret_present" "$signer_secret_hex_valid" "$CHECK_FILE" "$REQUIRED_RUNTIME_MODE" "$PRIMARY_SIGNER_PROFILE" "$SECONDARY_SIGNER_PROFILE" "$PRIMARY_SIGNER_SECRET_ENV" "$SECONDARY_SIGNER_SECRET_ENV" "$REQUIRED_SECRET_HEX_LENGTH" <<'PY'
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
