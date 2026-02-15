#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-live-deployment-preflight-summary.json"
RUNTIME_MODE="kolme-live"
SIGNER_PROFILE=""
MAX_SECONDS=12
REQUIRED_APPROVALS=2
RECEIVED_APPROVALS=0
QUORUM_EVIDENCE_FILE=""
CUSTODY_EVIDENCE_FILE=""
SIGNER_PROVENANCE_FILE=""
SIGNER_KEY_SOURCE_CONTRACT_VERSION="v1"
SIGNER_KEY_SOURCE="managed-external"
SIGNER_ROTATION_EPOCH=1
SIGNER_PREVIOUS_ROTATION_EPOCH=1
SIGNER_ROTATION_FRESHNESS_MAX_DELTA=2
MIN_PRODUCTION_REQUIRED_APPROVALS=2

REQUIRED_RUNTIME_MODE="kolme-live"
SIGNER_PROFILE_SELECTOR_ENV="KAMN_KOLME_LIVE_SIGNER_PROFILE"
PRIMARY_SIGNER_PROFILE="ops-primary"
SECONDARY_SIGNER_PROFILE="ops-secondary"
PRIMARY_SIGNER_SECRET_ENV="KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
SECONDARY_SIGNER_SECRET_ENV="KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
FALLBACK_SIGNER_SECRET_ENV="KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"
FALLBACK_SIGNER_SECRET_REMEDIATION="unset ${FALLBACK_SIGNER_SECRET_ENV}"
REQUIRED_SECRET_HEX_LENGTH=64
SIGNER_KEY_SOURCE_CONTRACT_VERSION_SUPPORTED="v1"
REQUIRED_PRODUCTION_SIGNER_KEY_SOURCE="managed-external"
RUNTIME_SIGNER_ATTESTATION_SCHEMA_VERSION="kamn.kolme.runtime-signer-attestation.v1"
RUNTIME_SIGNER_DRIFT_TELEMETRY_SCHEMA_VERSION="kamn.kolme.runtime-signer-drift-telemetry.v1"
RUNTIME_SIGNER_DRIFT_THRESHOLDS_SCHEMA_VERSION="kamn.kolme.runtime-signer-drift-thresholds.v1"
QUORUM_EVIDENCE_SCHEMA_VERSION="$RUNTIME_SIGNER_ATTESTATION_SCHEMA_VERSION"

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
    --quorum-evidence-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --quorum-evidence-file" >&2
        exit 1
      fi
      QUORUM_EVIDENCE_FILE="$2"
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
    --signer-provenance-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --signer-provenance-file" >&2
        exit 1
      fi
      SIGNER_PROVENANCE_FILE="$2"
      shift 2
      ;;
    --signer-key-source-contract-version)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --signer-key-source-contract-version" >&2
        exit 1
      fi
      SIGNER_KEY_SOURCE_CONTRACT_VERSION="$2"
      shift 2
      ;;
    --signer-key-source)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --signer-key-source" >&2
        exit 1
      fi
      SIGNER_KEY_SOURCE="$2"
      shift 2
      ;;
    --signer-rotation-epoch)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --signer-rotation-epoch" >&2
        exit 1
      fi
      SIGNER_ROTATION_EPOCH="$2"
      shift 2
      ;;
    --signer-previous-rotation-epoch)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --signer-previous-rotation-epoch" >&2
        exit 1
      fi
      SIGNER_PREVIOUS_ROTATION_EPOCH="$2"
      shift 2
      ;;
    --signer-rotation-freshness-max-delta)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --signer-rotation-freshness-max-delta" >&2
        exit 1
      fi
      SIGNER_ROTATION_FRESHNESS_MAX_DELTA="$2"
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
  --quorum-evidence-file <path>       Required quorum evidence bundle in run mode.
  --custody-evidence-file <path>      Required signer custody evidence file in run mode.
  --signer-provenance-file <path>     Required signer provenance evidence file in run mode.
  --signer-key-source-contract-version <value>
                                      Signer key-source contract version (default: v1).
  --signer-key-source <value>         Signer key-source marker (env-local|managed-external).
  --signer-rotation-epoch <n>         Current signer rotation epoch marker.
  --signer-previous-rotation-epoch <n>
                                      Previous signer rotation epoch marker.
  --signer-rotation-freshness-max-delta <n>
                                      Maximum allowed rotation epoch delta before stale rejection.
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

if ! [[ "$SIGNER_ROTATION_EPOCH" =~ ^[0-9]+$ ]] || [ "$SIGNER_ROTATION_EPOCH" -le 0 ]; then
  echo "signer-rotation-epoch must be a positive integer" >&2
  exit 1
fi

if ! [[ "$SIGNER_PREVIOUS_ROTATION_EPOCH" =~ ^[0-9]+$ ]] || [ "$SIGNER_PREVIOUS_ROTATION_EPOCH" -le 0 ]; then
  echo "signer-previous-rotation-epoch must be a positive integer" >&2
  exit 1
fi

if ! [[ "$SIGNER_ROTATION_FRESHNESS_MAX_DELTA" =~ ^[0-9]+$ ]]; then
  echo "signer-rotation-freshness-max-delta must be a non-negative integer" >&2
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
fallback_private_key_command="fallback signer secret env must remain unset for production signer profiles (remediation: ${FALLBACK_SIGNER_SECRET_REMEDIATION})"
signer_quorum_command="received approvals must satisfy required approvals threshold"
quorum_evidence_command="quorum evidence bundle must satisfy schema, signer uniqueness, threshold, and custody digest match"
custody_evidence_command="signer custody evidence file and sha256 marker must be present"
signer_provenance_command="signer provenance evidence file and sha256 marker must be present"
signer_rotation_freshness_command="signer rotation metadata must satisfy freshness threshold"

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
signer_secret_present="false"
signer_secret_hex_valid="false"
fallback_signer_secret_present="false"
quorum_evidence_present="false"
quorum_evidence_sha256=""
quorum_evidence_sha256_valid="false"
quorum_evidence_schema_valid="false"
quorum_evidence_approval_count=0
quorum_evidence_signers_unique="false"
quorum_evidence_matches_threshold="false"
quorum_evidence_custody_sha256_match="false"
quorum_evidence_signer_roles_present="false"
quorum_evidence_signer_roles_valid="false"
quorum_evidence_rotation_metadata_present="false"
quorum_evidence_rotation_metadata_valid="false"
custody_evidence_present="false"
custody_evidence_sha256=""
custody_evidence_sha256_valid="false"
signer_provenance_present="false"
signer_provenance_sha256=""
signer_provenance_sha256_valid="false"
signer_rotation_delta_epochs=0
signer_rotation_fresh="false"
runtime_signer_attestation_approved_signers_csv=""
runtime_signer_attestation_profile_approved="false"

record_check "runtime_mode_contract" "$runtime_mode_command" "planned" "not_run"
record_check "signer_profile_contract" "$signer_profile_command" "planned" "not_run"
record_check "signer_secret_contract" "$signer_secret_command" "planned" "not_run"
record_check "fallback_private_key_contract" "$fallback_private_key_command" "planned" "not_run"
record_check "signer_quorum_contract" "$signer_quorum_command" "planned" "not_run"
record_check "quorum_evidence_contract" "$quorum_evidence_command" "planned" "not_run"
record_check "custody_evidence_contract" "$custody_evidence_command" "planned" "not_run"
record_check "signer_provenance_contract" "$signer_provenance_command" "planned" "not_run"
record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if [ "$RUNTIME_MODE" != "$REQUIRED_RUNTIME_MODE" ]; then
    record_check "runtime_mode_contract" "$runtime_mode_command" "fail" "runtime_mode_mismatch"
    record_check "signer_profile_contract" "$signer_profile_command" "skipped" "runtime_mode_mismatch"
    record_check "signer_secret_contract" "$signer_secret_command" "skipped" "runtime_mode_mismatch"
    record_check "fallback_private_key_contract" "$fallback_private_key_command" "skipped" "runtime_mode_mismatch"
    record_check "signer_quorum_contract" "$signer_quorum_command" "skipped" "runtime_mode_mismatch"
    record_check "quorum_evidence_contract" "$quorum_evidence_command" "skipped" "runtime_mode_mismatch"
    record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "runtime_mode_mismatch"
    record_check "signer_provenance_contract" "$signer_provenance_command" "skipped" "runtime_mode_mismatch"
    record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "skipped" "runtime_mode_mismatch"
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
      record_check "quorum_evidence_contract" "$quorum_evidence_command" "skipped" "signer_profile_invalid"
      record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "signer_profile_invalid"
      record_check "signer_provenance_contract" "$signer_provenance_command" "skipped" "signer_profile_invalid"
      record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "skipped" "signer_profile_invalid"
      overall_status="fail"
      reason_code="checkpoint_failed_signer_profile_contract"
    else
      record_check "signer_profile_contract" "$signer_profile_command" "pass" "signer_profile_validated"

      fallback_signer_secret_value="${!FALLBACK_SIGNER_SECRET_ENV:-}"
      if [ -n "$fallback_signer_secret_value" ]; then
        fallback_signer_secret_present="true"
      fi

      if [ "$fallback_signer_secret_present" = "true" ]; then
        echo "fallback signer secret env must not be set: $FALLBACK_SIGNER_SECRET_ENV (remediation: $FALLBACK_SIGNER_SECRET_REMEDIATION)" >&2
        record_check "fallback_private_key_contract" "$fallback_private_key_command" "fail" "fallback_signer_secret_present_violation"
        record_check "signer_secret_contract" "$signer_secret_command" "skipped" "fallback_signer_secret_present_violation"
        record_check "signer_quorum_contract" "$signer_quorum_command" "skipped" "fallback_signer_secret_present_violation"
        record_check "quorum_evidence_contract" "$quorum_evidence_command" "skipped" "fallback_signer_secret_present_violation"
        record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "fallback_signer_secret_present_violation"
        record_check "signer_provenance_contract" "$signer_provenance_command" "skipped" "fallback_signer_secret_present_violation"
        record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "skipped" "fallback_signer_secret_present_violation"
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
          record_check "quorum_evidence_contract" "$quorum_evidence_command" "skipped" "signer_secret_missing"
          record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "signer_secret_missing"
          record_check "signer_provenance_contract" "$signer_provenance_command" "skipped" "signer_secret_missing"
          record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "skipped" "signer_secret_missing"
          overall_status="fail"
          reason_code="checkpoint_failed_signer_secret_contract"
        elif [ "$signer_secret_hex_valid" != "true" ]; then
          echo "signer secret env must be ${REQUIRED_SECRET_HEX_LENGTH} hex characters: $selected_signer_secret_env" >&2
          record_check "signer_secret_contract" "$signer_secret_command" "fail" "signer_secret_invalid_hex"
          record_check "signer_quorum_contract" "$signer_quorum_command" "skipped" "signer_secret_invalid_hex"
          record_check "quorum_evidence_contract" "$quorum_evidence_command" "skipped" "signer_secret_invalid_hex"
          record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "signer_secret_invalid_hex"
          record_check "signer_provenance_contract" "$signer_provenance_command" "skipped" "signer_secret_invalid_hex"
          record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "skipped" "signer_secret_invalid_hex"
          overall_status="fail"
          reason_code="checkpoint_failed_signer_secret_contract"
        else
          record_check "signer_secret_contract" "$signer_secret_command" "pass" "signer_secret_validated"
          if [ "$REQUIRED_APPROVALS" -lt "$MIN_PRODUCTION_REQUIRED_APPROVALS" ]; then
            echo "required approvals must be at least ${MIN_PRODUCTION_REQUIRED_APPROVALS} for production signer profiles: required=$REQUIRED_APPROVALS" >&2
            record_check "signer_quorum_contract" "$signer_quorum_command" "fail" "signer_quorum_minimum_not_met"
            record_check "quorum_evidence_contract" "$quorum_evidence_command" "skipped" "signer_quorum_minimum_not_met"
            record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "signer_quorum_minimum_not_met"
            record_check "signer_provenance_contract" "$signer_provenance_command" "skipped" "signer_quorum_minimum_not_met"
            record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "skipped" "signer_quorum_minimum_not_met"
            overall_status="fail"
            reason_code="checkpoint_failed_signer_quorum_contract"
          elif [ "$RECEIVED_APPROVALS" -lt "$REQUIRED_APPROVALS" ]; then
            echo "signer quorum approvals below required threshold: required=$REQUIRED_APPROVALS received=$RECEIVED_APPROVALS" >&2
            record_check "signer_quorum_contract" "$signer_quorum_command" "fail" "signer_quorum_shortfall"
            record_check "quorum_evidence_contract" "$quorum_evidence_command" "skipped" "signer_quorum_shortfall"
            record_check "custody_evidence_contract" "$custody_evidence_command" "skipped" "signer_quorum_shortfall"
            record_check "signer_provenance_contract" "$signer_provenance_command" "skipped" "signer_quorum_shortfall"
            record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "skipped" "signer_quorum_shortfall"
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
              record_check "quorum_evidence_contract" "$quorum_evidence_command" "skipped" "custody_evidence_missing"
              record_check "signer_provenance_contract" "$signer_provenance_command" "skipped" "custody_evidence_missing"
              record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "skipped" "custody_evidence_missing"
              overall_status="fail"
              reason_code="checkpoint_failed_custody_evidence_contract"
            elif [ "$custody_evidence_sha256_valid" != "true" ]; then
              echo "signer custody evidence sha256 marker is invalid: $CUSTODY_EVIDENCE_FILE" >&2
              record_check "custody_evidence_contract" "$custody_evidence_command" "fail" "custody_evidence_sha256_invalid"
              record_check "quorum_evidence_contract" "$quorum_evidence_command" "skipped" "custody_evidence_sha256_invalid"
              record_check "signer_provenance_contract" "$signer_provenance_command" "skipped" "custody_evidence_sha256_invalid"
              record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "skipped" "custody_evidence_sha256_invalid"
              overall_status="fail"
              reason_code="checkpoint_failed_custody_evidence_contract"
            else
              record_check "custody_evidence_contract" "$custody_evidence_command" "pass" "custody_evidence_validated"

              quorum_evidence_reason=""
              quorum_evidence_message=""
              if [ -n "$QUORUM_EVIDENCE_FILE" ] && [ -f "$QUORUM_EVIDENCE_FILE" ]; then
                quorum_evidence_present="true"
                quorum_evidence_sha256="$(sha256sum "$QUORUM_EVIDENCE_FILE" | awk '{print $1}')"
                if [[ "$quorum_evidence_sha256" =~ ^[0-9a-fA-F]{64}$ ]]; then
                  quorum_evidence_sha256_valid="true"
                fi
              fi

              if [ "$quorum_evidence_present" != "true" ]; then
                quorum_evidence_reason="quorum_evidence_missing"
                quorum_evidence_message="signer quorum evidence file is required for selected profile"
              elif [ "$quorum_evidence_sha256_valid" != "true" ]; then
                quorum_evidence_reason="quorum_evidence_sha256_invalid"
                quorum_evidence_message="signer quorum evidence sha256 marker is invalid: $QUORUM_EVIDENCE_FILE"
              else
                quorum_evidence_parse_output="$(
                  python3 "$ROOT_DIR/scripts/kolme/contracts/local_kolme_live_deployment_preflight_quorum_evidence_parse.py" "$QUORUM_EVIDENCE_FILE" "$QUORUM_EVIDENCE_SCHEMA_VERSION" "$REQUIRED_APPROVALS" "$RECEIVED_APPROVALS" "$custody_evidence_sha256" "$SIGNER_PROFILE"
                )"

                parsed_quorum_reason_code=""
                while IFS='=' read -r key value; do
                  case "$key" in
                    schema_valid)
                      if [ "$value" = "True" ] || [ "$value" = "true" ]; then
                        quorum_evidence_schema_valid="true"
                      else
                        quorum_evidence_schema_valid="false"
                      fi
                      ;;
                    approval_count)
                      if [[ "$value" =~ ^[0-9]+$ ]]; then
                        quorum_evidence_approval_count="$value"
                      else
                        quorum_evidence_approval_count=0
                      fi
                      ;;
                    signers_unique)
                      if [ "$value" = "True" ] || [ "$value" = "true" ]; then
                        quorum_evidence_signers_unique="true"
                      else
                        quorum_evidence_signers_unique="false"
                      fi
                      ;;
                    matches_threshold)
                      if [ "$value" = "True" ] || [ "$value" = "true" ]; then
                        quorum_evidence_matches_threshold="true"
                      else
                        quorum_evidence_matches_threshold="false"
                      fi
                      ;;
                    custody_sha256_match)
                      if [ "$value" = "True" ] || [ "$value" = "true" ]; then
                        quorum_evidence_custody_sha256_match="true"
                      else
                        quorum_evidence_custody_sha256_match="false"
                      fi
                      ;;
                    signer_roles_present)
                      if [ "$value" = "True" ] || [ "$value" = "true" ]; then
                        quorum_evidence_signer_roles_present="true"
                      else
                        quorum_evidence_signer_roles_present="false"
                      fi
                      ;;
                    signer_roles_valid)
                      if [ "$value" = "True" ] || [ "$value" = "true" ]; then
                        quorum_evidence_signer_roles_valid="true"
                      else
                        quorum_evidence_signer_roles_valid="false"
                      fi
                      ;;
                    rotation_metadata_present)
                      if [ "$value" = "True" ] || [ "$value" = "true" ]; then
                        quorum_evidence_rotation_metadata_present="true"
                      else
                        quorum_evidence_rotation_metadata_present="false"
                      fi
                      ;;
                    rotation_metadata_valid)
                      if [ "$value" = "True" ] || [ "$value" = "true" ]; then
                        quorum_evidence_rotation_metadata_valid="true"
                      else
                        quorum_evidence_rotation_metadata_valid="false"
                      fi
                      ;;
                    profile_approved)
                      if [ "$value" = "True" ] || [ "$value" = "true" ]; then
                        runtime_signer_attestation_profile_approved="true"
                      else
                        runtime_signer_attestation_profile_approved="false"
                      fi
                      ;;
                    approved_signers_csv)
                      runtime_signer_attestation_approved_signers_csv="$value"
                      ;;
                    reason_code)
                      parsed_quorum_reason_code="$value"
                      ;;
                  esac
                done <<<"$quorum_evidence_parse_output"

                if [ "$parsed_quorum_reason_code" != "ok" ]; then
                  quorum_evidence_reason="$parsed_quorum_reason_code"
                  if [ "$parsed_quorum_reason_code" = "runtime_signer_attestation_schema_invalid" ]; then
                    quorum_evidence_message="runtime signer attestation schema is invalid: $QUORUM_EVIDENCE_FILE"
                  elif [ "$parsed_quorum_reason_code" = "runtime_signer_attestation_approved_signers_not_unique" ]; then
                    quorum_evidence_message="runtime signer attestation approved_signers must be unique: $QUORUM_EVIDENCE_FILE"
                  elif [ "$parsed_quorum_reason_code" = "quorum_evidence_signer_roles_missing" ]; then
                    quorum_evidence_message="runtime signer attestation signer_roles metadata must include all approved_signers"
                  elif [ "$parsed_quorum_reason_code" = "quorum_evidence_signer_roles_invalid" ]; then
                    quorum_evidence_message="runtime signer attestation signer_roles metadata contains invalid role values"
                  elif [ "$parsed_quorum_reason_code" = "quorum_evidence_rotation_metadata_missing" ]; then
                    quorum_evidence_message="runtime signer attestation signer_rotation_epochs metadata must include all approved_signers"
                  elif [ "$parsed_quorum_reason_code" = "quorum_evidence_rotation_metadata_invalid" ]; then
                    quorum_evidence_message="runtime signer attestation signer_rotation_epochs metadata must contain positive integer epochs"
                  elif [ "$parsed_quorum_reason_code" = "runtime_signer_attestation_quorum_shortfall" ]; then
                    quorum_evidence_message="runtime signer attestation approvals must satisfy required approvals threshold"
                  elif [ "$parsed_quorum_reason_code" = "runtime_signer_attestation_profile_not_approved" ]; then
                    quorum_evidence_message="runtime signer attestation approved_signers must include signer profile: $SIGNER_PROFILE"
                  elif [ "$parsed_quorum_reason_code" = "quorum_evidence_custody_sha256_mismatch" ]; then
                    quorum_evidence_message="signer quorum evidence custody digest must match custody evidence sha256"
                  else
                    quorum_evidence_reason="runtime_signer_attestation_schema_invalid"
                    quorum_evidence_message="runtime signer attestation schema is invalid: $QUORUM_EVIDENCE_FILE"
                  fi
                fi
              fi

              if [ -n "$quorum_evidence_reason" ]; then
                echo "$quorum_evidence_message" >&2
                record_check "quorum_evidence_contract" "$quorum_evidence_command" "fail" "$quorum_evidence_reason"
                record_check "signer_provenance_contract" "$signer_provenance_command" "skipped" "$quorum_evidence_reason"
                record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "skipped" "$quorum_evidence_reason"
                overall_status="fail"
                reason_code="checkpoint_failed_quorum_evidence_contract"
              else
                record_check "quorum_evidence_contract" "$quorum_evidence_command" "pass" "quorum_evidence_validated"

                signer_provenance_reason=""
                signer_provenance_message=""
                if [ "$SIGNER_KEY_SOURCE_CONTRACT_VERSION" != "$SIGNER_KEY_SOURCE_CONTRACT_VERSION_SUPPORTED" ]; then
                  signer_provenance_reason="signer_key_source_contract_version_mismatch"
                  signer_provenance_message="signer key source contract version must remain ${SIGNER_KEY_SOURCE_CONTRACT_VERSION_SUPPORTED}: $SIGNER_KEY_SOURCE_CONTRACT_VERSION"
                elif [ "$SIGNER_KEY_SOURCE" != "env-local" ] && [ "$SIGNER_KEY_SOURCE" != "managed-external" ]; then
                  signer_provenance_reason="signer_key_source_invalid"
                  signer_provenance_message="signer key source is unsupported: $SIGNER_KEY_SOURCE"
                elif (
                  [ "$SIGNER_PROFILE" = "$PRIMARY_SIGNER_PROFILE" ] ||
                  [ "$SIGNER_PROFILE" = "$SECONDARY_SIGNER_PROFILE" ]
                ) && [ "$SIGNER_KEY_SOURCE" != "$REQUIRED_PRODUCTION_SIGNER_KEY_SOURCE" ]; then
                  signer_provenance_reason="signer_key_source_production_managed_external_required"
                  signer_provenance_message="production signer profiles require signer key source $REQUIRED_PRODUCTION_SIGNER_KEY_SOURCE: profile=$SIGNER_PROFILE source=$SIGNER_KEY_SOURCE"
                fi

                if [ -z "$signer_provenance_reason" ]; then
                  if [ -n "$SIGNER_PROVENANCE_FILE" ] && [ -f "$SIGNER_PROVENANCE_FILE" ]; then
                    signer_provenance_present="true"
                    signer_provenance_sha256="$(sha256sum "$SIGNER_PROVENANCE_FILE" | awk '{print $1}')"
                    if [[ "$signer_provenance_sha256" =~ ^[0-9a-fA-F]{64}$ ]]; then
                      signer_provenance_sha256_valid="true"
                    fi
                  fi
                  if [ "$signer_provenance_present" != "true" ]; then
                    signer_provenance_reason="signer_provenance_missing"
                    signer_provenance_message="signer provenance evidence file is required for selected profile"
                  elif [ "$signer_provenance_sha256_valid" != "true" ]; then
                    signer_provenance_reason="signer_provenance_sha256_invalid"
                    signer_provenance_message="signer provenance evidence sha256 marker is invalid: $SIGNER_PROVENANCE_FILE"
                  fi
                fi

                if [ -n "$signer_provenance_reason" ]; then
                  echo "$signer_provenance_message" >&2
                  record_check "signer_provenance_contract" "$signer_provenance_command" "fail" "$signer_provenance_reason"
                  record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "skipped" "$signer_provenance_reason"
                  overall_status="fail"
                  reason_code="checkpoint_failed_signer_provenance_contract"
                else
                  record_check "signer_provenance_contract" "$signer_provenance_command" "pass" "signer_provenance_validated"

                  signer_rotation_delta_epochs="$(( SIGNER_ROTATION_EPOCH - SIGNER_PREVIOUS_ROTATION_EPOCH ))"
                  signer_rotation_reason=""
                  signer_rotation_message=""
                  if [ "$signer_rotation_delta_epochs" -lt 0 ]; then
                    signer_rotation_reason="signer_rotation_epoch_invalid"
                    signer_rotation_message="signer rotation epoch must be greater than or equal to previous rotation epoch"
                  elif [ "$signer_rotation_delta_epochs" -gt "$SIGNER_ROTATION_FRESHNESS_MAX_DELTA" ]; then
                    signer_rotation_reason="signer_rotation_epoch_stale"
                    signer_rotation_message="signer rotation metadata exceeded freshness threshold: delta=$signer_rotation_delta_epochs max=$SIGNER_ROTATION_FRESHNESS_MAX_DELTA"
                  fi

                  if [ -n "$signer_rotation_reason" ]; then
                    echo "$signer_rotation_message" >&2
                    signer_rotation_fresh="false"
                    record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "fail" "$signer_rotation_reason"
                    overall_status="fail"
                    reason_code="checkpoint_failed_signer_rotation_freshness_contract"
                  else
                    signer_rotation_fresh="true"
                    record_check "signer_rotation_freshness_contract" "$signer_rotation_freshness_command" "pass" "signer_rotation_freshness_validated"
                    reason_code="deployment_preflight_passed"
                  fi
                fi
              fi
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
    overall_status="fail"
    reason_code="preflight_budget_exceeded"
  fi
fi

python3 "$ROOT_DIR/scripts/kolme/contracts/local_kolme_live_deployment_preflight_summary.py" "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$RUNTIME_MODE" "$SIGNER_PROFILE_SELECTOR_ENV" "$SIGNER_PROFILE" "$selected_signer_secret_env" "$FALLBACK_SIGNER_SECRET_ENV" "$signer_secret_present" "$fallback_signer_secret_present" "$signer_secret_hex_valid" "$CHECK_FILE" "$REQUIRED_RUNTIME_MODE" "$PRIMARY_SIGNER_PROFILE" "$SECONDARY_SIGNER_PROFILE" "$PRIMARY_SIGNER_SECRET_ENV" "$SECONDARY_SIGNER_SECRET_ENV" "$REQUIRED_SECRET_HEX_LENGTH" "$REQUIRED_APPROVALS" "$RECEIVED_APPROVALS" "$QUORUM_EVIDENCE_FILE" "$quorum_evidence_present" "$quorum_evidence_sha256" "$quorum_evidence_sha256_valid" "$quorum_evidence_schema_valid" "$quorum_evidence_approval_count" "$quorum_evidence_signers_unique" "$quorum_evidence_matches_threshold" "$quorum_evidence_custody_sha256_match" "$quorum_evidence_signer_roles_present" "$quorum_evidence_signer_roles_valid" "$quorum_evidence_rotation_metadata_present" "$quorum_evidence_rotation_metadata_valid" "$QUORUM_EVIDENCE_SCHEMA_VERSION" "$CUSTODY_EVIDENCE_FILE" "$custody_evidence_present" "$custody_evidence_sha256" "$custody_evidence_sha256_valid" "$SIGNER_PROVENANCE_FILE" "$signer_provenance_present" "$signer_provenance_sha256" "$signer_provenance_sha256_valid" "$SIGNER_KEY_SOURCE_CONTRACT_VERSION" "$SIGNER_KEY_SOURCE" "$SIGNER_KEY_SOURCE_CONTRACT_VERSION_SUPPORTED" "$SIGNER_ROTATION_EPOCH" "$SIGNER_PREVIOUS_ROTATION_EPOCH" "$SIGNER_ROTATION_FRESHNESS_MAX_DELTA" "$signer_rotation_delta_epochs" "$signer_rotation_fresh" "$RUNTIME_SIGNER_ATTESTATION_SCHEMA_VERSION" "$runtime_signer_attestation_approved_signers_csv" "$runtime_signer_attestation_profile_approved" "$RUNTIME_SIGNER_DRIFT_TELEMETRY_SCHEMA_VERSION" "$RUNTIME_SIGNER_DRIFT_THRESHOLDS_SCHEMA_VERSION"

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "ci_fast_gate_eligible=true"
echo "signer_key_source=$SIGNER_KEY_SOURCE"
echo "signer_rotation_delta_epochs=$signer_rotation_delta_epochs"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
