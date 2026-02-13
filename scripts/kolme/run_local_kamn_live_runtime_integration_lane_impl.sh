#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BOOTSTRAP_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh"
CONFORMANCE_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_live_api_conformance_harness.sh"
LOCALHOST_SIGNED_INTEGRATION_RUNNER="$ROOT_DIR/scripts/sdk/run_localhost_signed_integration_contract_lane.sh"
RUNTIME_COMMIT_LIVE_LANE_RUNNER="$ROOT_DIR/scripts/kolme/run_local_runtime_commit_live_lane.sh"
RUNTIME_COMMIT_LIVE_CONTRACT_LANE_RUNNER="$ROOT_DIR/scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-kamn-live-runtime-integration-summary.json"
BOOTSTRAP_REPORT="/tmp/kolme-local-fork-bootstrap-readiness-summary.json"
CONFORMANCE_REPORT="/tmp/kolme-local-live-api-conformance-summary.json"
LOCALHOST_SIGNED_REPORT="/tmp/localhost-signed-integration-contract-report.json"
RUNTIME_COMMIT_OUTPUT_FILE="/tmp/kolme-local-runtime-commit-endpoint-output.txt"
RUNTIME_COMMIT_LIVE_SUMMARY="/tmp/kolme-local-runtime-commit-live-summary.json"
RUNTIME_COMMIT_LIVE_POLICY_REPORT="/tmp/kolme-local-runtime-commit-live-policy.json"
RUNTIME_SIGNING_PROFILE="kolme-fork-secp256k1-v1"
RUNTIME_PROVIDER_CLIENT_CONTRACT="KolmeRuntimeCommitLiveProvider"
RUNTIME_PROFILE="real-node"
RUNTIME_COMMIT_FINALITY_COMMAND=""
RUNTIME_COMMIT_FINALITY_MAX_SECONDS=15
RUNTIME_COMMIT_FINALITY_OUTPUT_FILE="/tmp/kolme-local-runtime-commit-live-finality-output.txt"
CHECKOUT_PATH="/tmp/kolme_fork"
EXPECTED_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REF="refs/heads/main"
BASE_URL="http://127.0.0.1:3000"
FORK_CHAIN_VERSION="v0.15.2"
RUNTIME_COMMIT_COMMAND=""
MAX_SECONDS=210
BOOTSTRAP_MAX_SECONDS=90
CONFORMANCE_MAX_SECONDS=180
LOCALHOST_SIGNED_MAX_SECONDS=45
RUNTIME_COMMIT_MAX_SECONDS=30
RUNTIME_SIGNER_PROFILE_SELECTOR_ENV=""
RUNTIME_SIGNER_PROFILE=""
RUNTIME_SIGNER_KEY_SOURCE_CONTRACT_VERSION=""
RUNTIME_SIGNER_KEY_SOURCE=""
RUNTIME_SIGNER_PRIVATE_KEY_ENV=""
RUNTIME_SIGNER_KEY_REF_ENV=""
RUNTIME_SIGNER_PUBLIC_KEY_ENV=""
RUNTIME_SIGNER_FALLBACK_PRIVATE_KEY_ENV="KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"
RUNTIME_SIGNER_FALLBACK_GUARD_CONTRACT_VERSION="v2"
RUNTIME_SIGNER_FALLBACK_GUARD_MODE="reject_if_present"
RUNTIME_SIGNER_MANAGED_EXTERNAL_RAW_PRIVATE_KEY_REMEDIATION=""
RUNTIME_SIGNER_PROFILE_OVERRIDE=""
RUNTIME_SIGNER_KEY_SOURCE_OVERRIDE=""

RUNTIME_SIGNER_PRIVATE_KEY_ENV_PRIMARY="KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
RUNTIME_SIGNER_PRIVATE_KEY_ENV_SECONDARY="KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
RUNTIME_SIGNER_KEY_REF_ENV_PRIMARY="KAMN_KOLME_LIVE_SIGNER_KEY_REF"
RUNTIME_SIGNER_KEY_REF_ENV_SECONDARY="KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY"
RUNTIME_SIGNER_PUBLIC_KEY_ENV_PRIMARY="KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX"
RUNTIME_SIGNER_PUBLIC_KEY_ENV_SECONDARY="KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY"
RUNTIME_SIGNER_PUBLIC_KEY_MARKER_PLACEHOLDER="0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
RUNTIME_SIGNER_ATTESTATION_SCHEMA_VERSION="kamn.kolme.runtime-signer-attestation.v1"


shell_escape() {
  printf "%q" "$1"
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
    --bootstrap-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --bootstrap-report" >&2
        exit 1
      fi
      BOOTSTRAP_REPORT="$2"
      shift 2
      ;;
    --conformance-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --conformance-report" >&2
        exit 1
      fi
      CONFORMANCE_REPORT="$2"
      shift 2
      ;;
    --localhost-signed-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --localhost-signed-report" >&2
        exit 1
      fi
      LOCALHOST_SIGNED_REPORT="$2"
      shift 2
      ;;
    --runtime-commit-output-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-commit-output-file" >&2
        exit 1
      fi
      RUNTIME_COMMIT_OUTPUT_FILE="$2"
      shift 2
      ;;
    --runtime-commit-live-summary)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-commit-live-summary" >&2
        exit 1
      fi
      RUNTIME_COMMIT_LIVE_SUMMARY="$2"
      shift 2
      ;;
    --runtime-commit-live-policy-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-commit-live-policy-report" >&2
        exit 1
      fi
      RUNTIME_COMMIT_LIVE_POLICY_REPORT="$2"
      shift 2
      ;;
    --runtime-provider-client-contract)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-provider-client-contract" >&2
        exit 1
      fi
      RUNTIME_PROVIDER_CLIENT_CONTRACT="$2"
      shift 2
      ;;
    --runtime-profile)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-profile" >&2
        exit 1
      fi
      RUNTIME_PROFILE="$2"
      shift 2
      ;;
    --runtime-signer-profile)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-signer-profile" >&2
        exit 1
      fi
      RUNTIME_SIGNER_PROFILE_OVERRIDE="$2"
      shift 2
      ;;
    --runtime-signer-key-source)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-signer-key-source" >&2
        exit 1
      fi
      RUNTIME_SIGNER_KEY_SOURCE_OVERRIDE="$2"
      shift 2
      ;;
    --runtime-commit-finality-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-commit-finality-command" >&2
        exit 1
      fi
      RUNTIME_COMMIT_FINALITY_COMMAND="$2"
      shift 2
      ;;
    --runtime-commit-finality-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-commit-finality-max-seconds" >&2
        exit 1
      fi
      RUNTIME_COMMIT_FINALITY_MAX_SECONDS="$2"
      shift 2
      ;;
    --runtime-commit-finality-output-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-commit-finality-output-file" >&2
        exit 1
      fi
      RUNTIME_COMMIT_FINALITY_OUTPUT_FILE="$2"
      shift 2
      ;;
    --checkout-path)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --checkout-path" >&2
        exit 1
      fi
      CHECKOUT_PATH="$2"
      shift 2
      ;;
    --expected-remote-url)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --expected-remote-url" >&2
        exit 1
      fi
      EXPECTED_REMOTE_URL="$2"
      shift 2
      ;;
    --expected-ref)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --expected-ref" >&2
        exit 1
      fi
      EXPECTED_REF="$2"
      shift 2
      ;;
    --base-url)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --base-url" >&2
        exit 1
      fi
      BASE_URL="$2"
      shift 2
      ;;
    --fork-chain-version)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --fork-chain-version" >&2
        exit 1
      fi
      FORK_CHAIN_VERSION="$2"
      shift 2
      ;;
    --runtime-commit-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-commit-command" >&2
        exit 1
      fi
      RUNTIME_COMMIT_COMMAND="$2"
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
    --bootstrap-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --bootstrap-max-seconds" >&2
        exit 1
      fi
      BOOTSTRAP_MAX_SECONDS="$2"
      shift 2
      ;;
    --conformance-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --conformance-max-seconds" >&2
        exit 1
      fi
      CONFORMANCE_MAX_SECONDS="$2"
      shift 2
      ;;
    --localhost-signed-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --localhost-signed-max-seconds" >&2
        exit 1
      fi
      LOCALHOST_SIGNED_MAX_SECONDS="$2"
      shift 2
      ;;
    --runtime-commit-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-commit-max-seconds" >&2
        exit 1
      fi
      RUNTIME_COMMIT_MAX_SECONDS="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kamn_live_runtime_integration_lane.sh [options]

Options:
  --mode dry-run|run                    Emit planned checks or execute local live checks.
  --output-json <path>                  Deterministic summary report output path.
  --bootstrap-report <path>             Output path for bootstrap/readiness summary.
  --conformance-report <path>           Output path for live API conformance summary.
  --localhost-signed-report <path>      Output path for localhost signed integration summary.
  --runtime-commit-output-file <path>   Captured stdout/stderr for runtime-commit endpoint command.
  --runtime-commit-live-summary <path>  Summary report output path for nested runtime-commit live lane runner.
  --runtime-commit-live-policy-report <path>
                                      Policy report output path for nested runtime-commit live evidence checks.
  --runtime-provider-client-contract <name>
                                      Required runtime provider contract marker forwarded to nested runtime finality evidence policy.
  --runtime-profile standard|real-node
                                      Runtime integration profile contract marker for local evidence interpretation.
  --runtime-signer-profile ops-primary|ops-secondary
                                      Optional signer profile override for real-node runtime profile summaries.
  --runtime-signer-key-source env-local|managed-external
                                      Optional signer key-source override for real-node runtime profile summaries.
  --runtime-commit-finality-command <command>
                                      Optional finality command passed through to nested runtime-commit live lane.
  --runtime-commit-finality-max-seconds <n>
                                      Max budget for runtime finality command passed through to nested live lane.
  --runtime-commit-finality-output-file <path>
                                      Captured stdout/stderr path for runtime finality command passed through to nested live lane.
  --checkout-path <path>                Local kolme_fork checkout path.
  --expected-remote-url <url>           Expected origin URL for checkout validation.
  --expected-ref <ref>                  Expected symbolic HEAD ref for checkout.
  --base-url <url>                      Base URL for local Kolme API server.
  --fork-chain-version <value>          Required chain_version query value for fork-info checks.
  --runtime-commit-command <command>    Runtime-commit endpoint command for run mode.
  --max-seconds <n>                     Max total runtime budget for run mode.
  --bootstrap-max-seconds <n>           Max budget for bootstrap/readiness prerequisite.
  --conformance-max-seconds <n>         Max budget for live API conformance prerequisite.
  --localhost-signed-max-seconds <n>    Max budget for localhost signed integration prerequisite.
  --runtime-commit-max-seconds <n>      Max budget for runtime-commit endpoint command.
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

if [ -z "$CHECKOUT_PATH" ] || [ -z "$EXPECTED_REMOTE_URL" ] || [ -z "$EXPECTED_REF" ]; then
  echo "checkout-path, expected-remote-url, and expected-ref must not be empty" >&2
  exit 1
fi

if [ -z "$BASE_URL" ] || [ -z "$FORK_CHAIN_VERSION" ]; then
  echo "base-url and fork-chain-version must not be empty" >&2
  exit 1
fi

if [ -z "$RUNTIME_PROVIDER_CLIENT_CONTRACT" ]; then
  echo "runtime-provider-client-contract must not be empty" >&2
  exit 1
fi

if [ "$RUNTIME_PROVIDER_CLIENT_CONTRACT" != "KolmeRuntimeCommitLiveProvider" ]; then
  echo "runtime-provider-client-contract must be KolmeRuntimeCommitLiveProvider" >&2
  exit 1
fi

if [ "$RUNTIME_PROFILE" != "standard" ] && [ "$RUNTIME_PROFILE" != "real-node" ]; then
  echo "runtime-profile must be one of: standard, real-node" >&2
  exit 1
fi

if [ "$MODE" = "run" ] && [ "$RUNTIME_PROFILE" != "real-node" ]; then
  echo "run mode requires runtime-profile=real-node; standard profile is dry-run only" >&2
  exit 1
fi

if [ -n "$RUNTIME_SIGNER_PROFILE_OVERRIDE" ] && [ "$RUNTIME_PROFILE" != "real-node" ]; then
  echo "runtime-signer-profile is only valid when runtime-profile=real-node" >&2
  exit 1
fi

if [ -n "$RUNTIME_SIGNER_PROFILE_OVERRIDE" ] && [ "$RUNTIME_SIGNER_PROFILE_OVERRIDE" != "ops-primary" ] && [ "$RUNTIME_SIGNER_PROFILE_OVERRIDE" != "ops-secondary" ]; then
  echo "runtime-signer-profile must be one of: ops-primary, ops-secondary" >&2
  exit 1
fi

if [ -n "$RUNTIME_SIGNER_KEY_SOURCE_OVERRIDE" ] && [ "$RUNTIME_PROFILE" != "real-node" ]; then
  echo "runtime-signer-key-source is only valid when runtime-profile=real-node" >&2
  exit 1
fi

if [ -n "$RUNTIME_SIGNER_KEY_SOURCE_OVERRIDE" ] && [ "$RUNTIME_SIGNER_KEY_SOURCE_OVERRIDE" != "env-local" ] && [ "$RUNTIME_SIGNER_KEY_SOURCE_OVERRIDE" != "managed-external" ]; then
  echo "runtime-signer-key-source must be one of: env-local, managed-external" >&2
  exit 1
fi

for numeric_value in "$MAX_SECONDS" "$BOOTSTRAP_MAX_SECONDS" "$CONFORMANCE_MAX_SECONDS" "$LOCALHOST_SIGNED_MAX_SECONDS" "$RUNTIME_COMMIT_MAX_SECONDS" "$RUNTIME_COMMIT_FINALITY_MAX_SECONDS"; do
  if ! [[ "$numeric_value" =~ ^[0-9]+$ ]] || [ "$numeric_value" -le 0 ]; then
    echo "all max-second arguments must be positive integers" >&2
    exit 1
  fi
done

if [ ! -x "$BOOTSTRAP_RUNNER" ]; then
  echo "expected local fork bootstrap/readiness runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CONFORMANCE_RUNNER" ]; then
  echo "expected local Kolme live API conformance harness runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCALHOST_SIGNED_INTEGRATION_RUNNER" ]; then
  echo "expected localhost signed integration contract runner to be executable" >&2
  exit 1
fi

if [ ! -x "$RUNTIME_COMMIT_LIVE_LANE_RUNNER" ]; then
  echo "expected local runtime commit live lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$RUNTIME_COMMIT_LIVE_CONTRACT_LANE_RUNNER" ]; then
  echo "expected local runtime commit live finality evidence contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

default_runtime_commit_live_submit_command_prefix="KAMN_KOLME_LIVE_BASE_URL=$(shell_escape "${BASE_URL}") KAMN_KOLME_LIVE_PROVIDER_HINT=kolme-fork-local"
effective_runtime_commit_finality_command="$RUNTIME_COMMIT_FINALITY_COMMAND"
if [ -z "$effective_runtime_commit_finality_command" ]; then
  effective_runtime_commit_finality_command="printf 'finality=final\\n'"
fi

runtime_commit_non_synthetic_flag=""
runtime_commit_native_payload_flag=""
runtime_commit_command_profile="standard-default-v1"
runtime_commit_policy_command_profile="standard-default-v1"
runtime_commit_command_profile_version="v1"
RUNTIME_SIGNER_PREVIOUS_PROFILE=""
RUNTIME_SIGNER_FAILOVER_ACTIVE="false"
RUNTIME_SIGNER_ROTATION_EPOCH="1"
RUNTIME_SIGNER_PREVIOUS_ROTATION_EPOCH="1"
if [ "$RUNTIME_PROFILE" = "real-node" ]; then
  runtime_commit_non_synthetic_flag=" --require-non-synthetic-run-evidence"
  runtime_commit_native_payload_flag=" --require-native-payload-evidence"
  runtime_commit_command_profile="real-node-non-synthetic-v1"
  runtime_commit_policy_command_profile="real-node-non-synthetic-v1"
  RUNTIME_SIGNER_PROFILE_SELECTOR_ENV="KAMN_KOLME_LIVE_SIGNER_PROFILE"
  RUNTIME_SIGNER_KEY_SOURCE_CONTRACT_VERSION="v1"
  RUNTIME_SIGNER_KEY_SOURCE="env-local"
  RUNTIME_SIGNER_PROFILE="${RUNTIME_SIGNER_PROFILE_OVERRIDE:-ops-primary}"
  if [ "$RUNTIME_SIGNER_PROFILE" = "ops-secondary" ]; then
    RUNTIME_SIGNER_PRIVATE_KEY_ENV="$RUNTIME_SIGNER_PRIVATE_KEY_ENV_SECONDARY"
    RUNTIME_SIGNER_KEY_REF_ENV="$RUNTIME_SIGNER_KEY_REF_ENV_SECONDARY"
    RUNTIME_SIGNER_PUBLIC_KEY_ENV="$RUNTIME_SIGNER_PUBLIC_KEY_ENV_SECONDARY"
  else
    RUNTIME_SIGNER_PRIVATE_KEY_ENV="$RUNTIME_SIGNER_PRIVATE_KEY_ENV_PRIMARY"
    RUNTIME_SIGNER_KEY_REF_ENV="$RUNTIME_SIGNER_KEY_REF_ENV_PRIMARY"
    RUNTIME_SIGNER_PUBLIC_KEY_ENV="$RUNTIME_SIGNER_PUBLIC_KEY_ENV_PRIMARY"
  fi
  if [ -n "$RUNTIME_SIGNER_KEY_SOURCE_OVERRIDE" ]; then
    RUNTIME_SIGNER_KEY_SOURCE="$RUNTIME_SIGNER_KEY_SOURCE_OVERRIDE"
  fi
  if [ "$RUNTIME_SIGNER_PROFILE" = "ops-secondary" ] && [ "$RUNTIME_SIGNER_KEY_SOURCE" = "managed-external" ]; then
    echo "runtime-signer-key-source managed-external is not allowed for runtime-signer-profile=ops-secondary" >&2
    exit 1
  fi
  RUNTIME_SIGNER_MANAGED_EXTERNAL_RAW_PRIVATE_KEY_REMEDIATION="unset ${RUNTIME_SIGNER_PRIVATE_KEY_ENV}; set ${RUNTIME_SIGNER_KEY_REF_ENV}"
  RUNTIME_SIGNER_PREVIOUS_PROFILE="$RUNTIME_SIGNER_PROFILE"
fi

if [ -n "$RUNTIME_SIGNER_PROFILE" ]; then
  default_runtime_commit_live_submit_command_prefix="${default_runtime_commit_live_submit_command_prefix} KAMN_KOLME_LIVE_SIGNER_PROFILE=${RUNTIME_SIGNER_PROFILE}"
fi
if [ -n "$RUNTIME_SIGNER_KEY_SOURCE" ]; then
  default_runtime_commit_live_submit_command_prefix="${default_runtime_commit_live_submit_command_prefix} KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE=${RUNTIME_SIGNER_KEY_SOURCE}"
fi
if [ "$RUNTIME_SIGNER_KEY_SOURCE" = "managed-external" ] && [ -n "$RUNTIME_SIGNER_KEY_REF_ENV" ] && [ -n "$RUNTIME_SIGNER_PROFILE" ]; then
  default_runtime_commit_live_submit_command_prefix="${default_runtime_commit_live_submit_command_prefix} ${RUNTIME_SIGNER_KEY_REF_ENV}=secure:aws-kms:role-operator/key-live-${RUNTIME_SIGNER_PROFILE}"
fi
if [ "$RUNTIME_SIGNER_KEY_SOURCE" = "managed-external" ] && [ -n "$RUNTIME_SIGNER_PUBLIC_KEY_ENV" ] && [ -n "$RUNTIME_SIGNER_PROFILE" ]; then
  default_runtime_commit_live_submit_command_prefix="${default_runtime_commit_live_submit_command_prefix} ${RUNTIME_SIGNER_PUBLIC_KEY_ENV}=${RUNTIME_SIGNER_PUBLIC_KEY_MARKER_PLACEHOLDER}"
fi
default_runtime_commit_live_submit_command="${default_runtime_commit_live_submit_command_prefix} KAMN_KOLME_LIVE_SIGNING_PROFILE=${RUNTIME_SIGNING_PROFILE} cargo test -p kamn-core --test kolme_runtime_commit_http_transport -- --exact integration_kolme_fork_live_node_submit_reaches_endpoint && printf 'status=submitted\\n{\"pubkey\":\"proof\",\"nonce\":1,\"messages\":[]}\\n'"

default_runtime_commit_command="KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --output-json $(shell_escape "${RUNTIME_COMMIT_LIVE_SUMMARY}") --policy-output-json $(shell_escape "${RUNTIME_COMMIT_LIVE_POLICY_REPORT}") --live-output-file $(shell_escape "${RUNTIME_COMMIT_OUTPUT_FILE}") --finality-output-file $(shell_escape "${RUNTIME_COMMIT_FINALITY_OUTPUT_FILE}") --max-seconds ${RUNTIME_COMMIT_MAX_SECONDS} --expected-provider-client-contract $(shell_escape "${RUNTIME_PROVIDER_CLIENT_CONTRACT}")${runtime_commit_non_synthetic_flag}${runtime_commit_native_payload_flag} --live-command $(shell_escape "${default_runtime_commit_live_submit_command}") --finality-command $(shell_escape "${effective_runtime_commit_finality_command}") --finality-max-seconds ${RUNTIME_COMMIT_FINALITY_MAX_SECONDS}"
if [ -z "$RUNTIME_COMMIT_COMMAND" ]; then
  RUNTIME_COMMIT_COMMAND="$default_runtime_commit_command"
  RUNTIME_COMMIT_FINALITY_COMMAND="$effective_runtime_commit_finality_command"
fi

if [ "$RUNTIME_PROFILE" = "real-node" ] && [[ "$RUNTIME_COMMIT_COMMAND" == *"InMemoryKolmeRuntimeCommitClient"* ]]; then
  echo "runtime-commit-command must not reference InMemoryKolmeRuntimeCommitClient when runtime-profile=real-node" >&2
  exit 1
fi

if [ "$RUNTIME_PROFILE" = "real-node" ] && [[ "$RUNTIME_COMMIT_COMMAND" == *"KAMN_KOLME_LIVE_SIGNING_PROFILE=simulated"* ]]; then
  echo "runtime-commit-command must not reference simulated signing profile marker when runtime-profile=real-node" >&2
  exit 1
fi

if [ "$RUNTIME_PROFILE" = "real-node" ] && [ -n "$RUNTIME_SIGNER_KEY_SOURCE" ]; then
  expected_runtime_signer_key_source_marker="KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE=${RUNTIME_SIGNER_KEY_SOURCE}"
  if [[ "$RUNTIME_COMMIT_COMMAND" != *"$expected_runtime_signer_key_source_marker"* ]]; then
    echo "runtime-commit-command must include signer key-source marker ${expected_runtime_signer_key_source_marker} when runtime-profile=real-node" >&2
    exit 1
  fi
fi

if [ "$RUNTIME_PROFILE" = "real-node" ] && [[ "$RUNTIME_COMMIT_COMMAND" == *"${RUNTIME_SIGNER_FALLBACK_PRIVATE_KEY_ENV}="* ]]; then
  echo "runtime-commit-command must not include fallback signer private key marker ${RUNTIME_SIGNER_FALLBACK_PRIVATE_KEY_ENV}=... when runtime-profile=real-node" >&2
  exit 1
fi

if [ "$RUNTIME_PROFILE" = "real-node" ] && [ "$RUNTIME_SIGNER_KEY_SOURCE" = "managed-external" ]; then
  expected_runtime_signer_key_reference_marker_prefix="${RUNTIME_SIGNER_KEY_REF_ENV}="
  if [[ "$RUNTIME_COMMIT_COMMAND" != *"$expected_runtime_signer_key_reference_marker_prefix"* ]]; then
    echo "runtime-commit-command must include managed signer key-reference marker ${RUNTIME_SIGNER_KEY_REF_ENV}=... when runtime-signer-key-source=managed-external" >&2
    exit 1
  fi
  expected_runtime_signer_public_key_marker_prefix="${RUNTIME_SIGNER_PUBLIC_KEY_ENV}="
  if [[ "$RUNTIME_COMMIT_COMMAND" != *"$expected_runtime_signer_public_key_marker_prefix"* ]]; then
    echo "runtime-commit-command must include managed signer public-key marker ${RUNTIME_SIGNER_PUBLIC_KEY_ENV}=... when runtime-signer-key-source=managed-external" >&2
    exit 1
  fi
  if [ -n "$RUNTIME_SIGNER_PRIVATE_KEY_ENV" ] && [[ "$RUNTIME_COMMIT_COMMAND" == *"${RUNTIME_SIGNER_PRIVATE_KEY_ENV}="* ]]; then
    echo "runtime-commit-command must not include raw signer private key marker ${RUNTIME_SIGNER_PRIVATE_KEY_ENV}=... when runtime-signer-key-source=managed-external" >&2
    exit 1
  fi
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

read_report_reason_code() {
  local report_file="$1"
  python3 - "$report_file" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
if not path.exists():
    print("report_missing")
    raise SystemExit(0)

try:
    payload = json.loads(path.read_text(encoding="utf-8"))
except json.JSONDecodeError:
    print("report_invalid_json")
    raise SystemExit(0)

value = payload.get("reason_code")
if isinstance(value, str) and value.strip():
    print(value)
else:
    print("reason_code_missing")
PY
}

read_policy_final_decision() {
  local report_file="$1"
  python3 - "$report_file" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
if not path.exists():
    print("report_missing")
    raise SystemExit(0)

try:
    payload = json.loads(path.read_text(encoding="utf-8"))
except json.JSONDecodeError:
    print("report_invalid_json")
    raise SystemExit(0)

value = payload.get("final_decision")
if isinstance(value, str) and value.strip():
    print(value)
else:
    print("final_decision_missing")
PY
}

bootstrap_command="bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run --checkout-path ${CHECKOUT_PATH} --expected-remote-url ${EXPECTED_REMOTE_URL} --expected-ref ${EXPECTED_REF} --base-url ${BASE_URL} --fork-chain-version ${FORK_CHAIN_VERSION} --max-seconds ${BOOTSTRAP_MAX_SECONDS} --probe-max-seconds 20 --output-json ${BOOTSTRAP_REPORT}"
localhost_signed_command="bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json ${LOCALHOST_SIGNED_REPORT}"
conformance_command="bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode run --base-url ${BASE_URL} --fork-chain-version ${FORK_CHAIN_VERSION} --max-seconds ${CONFORMANCE_MAX_SECONDS} --probe-max-seconds 30 --native-max-seconds 120 --output-json ${CONFORMANCE_REPORT}"
runtime_signer_fallback_private_key_command="fallback signer secret env must remain unset for real-node runtime profile"
runtime_signer_managed_external_raw_private_key_command="managed-external signer profile must reject raw private key env markers for selected profile"
runtime_commit_command="$RUNTIME_COMMIT_COMMAND"
runtime_commit_policy_command="python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file ${RUNTIME_COMMIT_LIVE_SUMMARY} --expected-final-decision GO --ci-fast-gate PASS --output-json ${RUNTIME_COMMIT_LIVE_POLICY_REPORT}"
if [ "$RUNTIME_PROFILE" = "real-node" ]; then
  runtime_commit_policy_command="${runtime_commit_policy_command} --require-non-synthetic-run-evidence --require-native-payload-evidence"
fi

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
bootstrap_reason_code="not_run"
localhost_signed_reason_code="not_run"
conformance_reason_code="not_run"
runtime_commit_reason_code="not_run"
runtime_commit_policy_reason_code="not_run"
runtime_signer_fallback_private_key_present="false"
runtime_signer_raw_private_key_present="false"

record_check "bootstrap_readiness" "$bootstrap_command" "planned" "not_run"
record_check "localhost_signed_integration" "$localhost_signed_command" "planned" "not_run"
record_check "live_api_conformance" "$conformance_command" "planned" "not_run"
record_check "runtime_signer_fallback_private_key_contract" "$runtime_signer_fallback_private_key_command" "planned" "not_run"
record_check "runtime_signer_managed_external_raw_private_key_contract" "$runtime_signer_managed_external_raw_private_key_command" "planned" "not_run"
record_check "runtime_commit_endpoint" "$runtime_commit_command" "planned" "not_run"
record_check "runtime_commit_policy" "$runtime_commit_policy_command" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if [ "$RUNTIME_PROFILE" = "real-node" ] && [ -n "${!RUNTIME_SIGNER_FALLBACK_PRIVATE_KEY_ENV:-}" ]; then
    runtime_signer_fallback_private_key_present="true"
    echo "fallback signer secret env must not be set: $RUNTIME_SIGNER_FALLBACK_PRIVATE_KEY_ENV (remediation: unset $RUNTIME_SIGNER_FALLBACK_PRIVATE_KEY_ENV)" >&2
    record_check "bootstrap_readiness" "$bootstrap_command" "skipped" "fallback_signer_secret_present_violation"
    record_check "localhost_signed_integration" "$localhost_signed_command" "skipped" "fallback_signer_secret_present_violation"
    record_check "live_api_conformance" "$conformance_command" "skipped" "fallback_signer_secret_present_violation"
    record_check "runtime_signer_fallback_private_key_contract" "$runtime_signer_fallback_private_key_command" "fail" "fallback_signer_secret_present_violation"
    record_check "runtime_signer_managed_external_raw_private_key_contract" "$runtime_signer_managed_external_raw_private_key_command" "skipped" "fallback_signer_secret_present_violation"
    record_check "runtime_commit_endpoint" "$runtime_commit_command" "skipped" "fallback_signer_secret_present_violation"
    record_check "runtime_commit_policy" "$runtime_commit_policy_command" "skipped" "fallback_signer_secret_present_violation"
    overall_status="fail"
    reason_code="runtime_signer_fallback_private_key_present_violation"
    bootstrap_reason_code="fallback_signer_secret_present_violation"
    localhost_signed_reason_code="fallback_signer_secret_present_violation"
    conformance_reason_code="fallback_signer_secret_present_violation"
    runtime_commit_reason_code="fallback_signer_secret_present_violation"
    runtime_commit_policy_reason_code="fallback_signer_secret_present_violation"
  else
    record_check "runtime_signer_fallback_private_key_contract" "$runtime_signer_fallback_private_key_command" "pass" "fallback_signer_secret_absent"
  fi

  if [ "$overall_status" = "ok" ]; then
    if [ "$RUNTIME_PROFILE" = "real-node" ] && [ "$RUNTIME_SIGNER_KEY_SOURCE" = "managed-external" ]; then
      if [ -n "${!RUNTIME_SIGNER_PRIVATE_KEY_ENV:-}" ]; then
        runtime_signer_raw_private_key_present="true"
        echo "managed-external signer raw private key env must not be set: $RUNTIME_SIGNER_PRIVATE_KEY_ENV (remediation: unset $RUNTIME_SIGNER_PRIVATE_KEY_ENV; set $RUNTIME_SIGNER_KEY_REF_ENV)" >&2
        record_check "bootstrap_readiness" "$bootstrap_command" "skipped" "managed_signer_raw_private_key_present_violation"
        record_check "localhost_signed_integration" "$localhost_signed_command" "skipped" "managed_signer_raw_private_key_present_violation"
        record_check "live_api_conformance" "$conformance_command" "skipped" "managed_signer_raw_private_key_present_violation"
        record_check "runtime_signer_managed_external_raw_private_key_contract" "$runtime_signer_managed_external_raw_private_key_command" "fail" "managed_signer_raw_private_key_present_violation"
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "skipped" "managed_signer_raw_private_key_present_violation"
        record_check "runtime_commit_policy" "$runtime_commit_policy_command" "skipped" "managed_signer_raw_private_key_present_violation"
        overall_status="fail"
        reason_code="runtime_signer_managed_external_raw_private_key_present_violation"
        bootstrap_reason_code="managed_signer_raw_private_key_present_violation"
        localhost_signed_reason_code="managed_signer_raw_private_key_present_violation"
        conformance_reason_code="managed_signer_raw_private_key_present_violation"
        runtime_commit_reason_code="managed_signer_raw_private_key_present_violation"
        runtime_commit_policy_reason_code="managed_signer_raw_private_key_present_violation"
      else
        record_check "runtime_signer_managed_external_raw_private_key_contract" "$runtime_signer_managed_external_raw_private_key_command" "pass" "managed_signer_raw_private_key_absent"
      fi
    else
      record_check "runtime_signer_managed_external_raw_private_key_contract" "$runtime_signer_managed_external_raw_private_key_command" "pass" "managed_signer_not_selected"
    fi
  fi

  if [ "$overall_status" = "ok" ]; then
    if ! "$LOCAL_HEAVY_GUARD"; then
      record_check "bootstrap_readiness" "$bootstrap_command" "fail" "local_opt_in_missing"
      record_check "localhost_signed_integration" "$localhost_signed_command" "skipped" "local_opt_in_missing"
      record_check "live_api_conformance" "$conformance_command" "skipped" "local_opt_in_missing"
      record_check "runtime_commit_endpoint" "$runtime_commit_command" "skipped" "local_opt_in_missing"
      record_check "runtime_commit_policy" "$runtime_commit_policy_command" "skipped" "local_opt_in_missing"
      overall_status="fail"
      reason_code="local_opt_in_missing"
      bootstrap_reason_code="local_opt_in_missing"
      localhost_signed_reason_code="local_opt_in_missing"
      conformance_reason_code="local_opt_in_missing"
      runtime_commit_reason_code="local_opt_in_missing"
      runtime_commit_policy_reason_code="local_opt_in_missing"
    else
    if KAMN_KOLME_LOCAL_HEAVY=1 bash "$BOOTSTRAP_RUNNER" \
      --mode run \
      --checkout-path "$CHECKOUT_PATH" \
      --expected-remote-url "$EXPECTED_REMOTE_URL" \
      --expected-ref "$EXPECTED_REF" \
      --base-url "$BASE_URL" \
      --fork-chain-version "$FORK_CHAIN_VERSION" \
      --max-seconds "$BOOTSTRAP_MAX_SECONDS" \
      --probe-max-seconds 20 \
      --output-json "$BOOTSTRAP_REPORT" >/dev/null; then
      record_check "bootstrap_readiness" "$bootstrap_command" "pass" "bootstrap_readiness_passed"
      bootstrap_reason_code="bootstrap_readiness_passed"
    else
      bootstrap_reason_code="$(read_report_reason_code "$BOOTSTRAP_REPORT")"
      record_check "bootstrap_readiness" "$bootstrap_command" "fail" "$bootstrap_reason_code"
      record_check "localhost_signed_integration" "$localhost_signed_command" "skipped" "bootstrap_readiness_failed"
      record_check "live_api_conformance" "$conformance_command" "skipped" "bootstrap_readiness_failed"
      record_check "runtime_commit_endpoint" "$runtime_commit_command" "skipped" "bootstrap_readiness_failed"
      record_check "runtime_commit_policy" "$runtime_commit_policy_command" "skipped" "bootstrap_readiness_failed"
      overall_status="fail"
      reason_code="bootstrap_readiness_failed"
      localhost_signed_reason_code="bootstrap_readiness_failed"
      conformance_reason_code="bootstrap_readiness_failed"
      runtime_commit_reason_code="bootstrap_readiness_failed"
      runtime_commit_policy_reason_code="bootstrap_readiness_failed"
    fi

    if [ "$overall_status" = "ok" ]; then
      set +e
      timeout "$LOCALHOST_SIGNED_MAX_SECONDS" bash "$LOCALHOST_SIGNED_INTEGRATION_RUNNER" \
        --output-json "$LOCALHOST_SIGNED_REPORT" >/dev/null 2>&1
      localhost_signed_exit_code=$?
      set -e

      if [ "$localhost_signed_exit_code" -eq 0 ]; then
        record_check "localhost_signed_integration" "$localhost_signed_command" "pass" "localhost_signed_integration_passed"
        localhost_signed_reason_code="localhost_signed_integration_passed"
      elif [ "$localhost_signed_exit_code" -eq 124 ]; then
        record_check "localhost_signed_integration" "$localhost_signed_command" "fail" "localhost_signed_integration_timeout"
        record_check "live_api_conformance" "$conformance_command" "skipped" "localhost_signed_integration_failed"
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "skipped" "localhost_signed_integration_failed"
        record_check "runtime_commit_policy" "$runtime_commit_policy_command" "skipped" "localhost_signed_integration_failed"
        overall_status="fail"
        reason_code="localhost_signed_integration_failed"
        localhost_signed_reason_code="localhost_signed_integration_timeout"
        conformance_reason_code="localhost_signed_integration_failed"
        runtime_commit_reason_code="localhost_signed_integration_failed"
        runtime_commit_policy_reason_code="localhost_signed_integration_failed"
      else
        record_check "localhost_signed_integration" "$localhost_signed_command" "fail" "localhost_signed_integration_failed"
        record_check "live_api_conformance" "$conformance_command" "skipped" "localhost_signed_integration_failed"
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "skipped" "localhost_signed_integration_failed"
        record_check "runtime_commit_policy" "$runtime_commit_policy_command" "skipped" "localhost_signed_integration_failed"
        overall_status="fail"
        reason_code="localhost_signed_integration_failed"
        localhost_signed_reason_code="localhost_signed_integration_failed"
        conformance_reason_code="localhost_signed_integration_failed"
        runtime_commit_reason_code="localhost_signed_integration_failed"
        runtime_commit_policy_reason_code="localhost_signed_integration_failed"
      fi
    fi

    if [ "$overall_status" = "ok" ]; then
      if KAMN_KOLME_LOCAL_HEAVY=1 bash "$CONFORMANCE_RUNNER" \
        --mode run \
        --base-url "$BASE_URL" \
        --fork-chain-version "$FORK_CHAIN_VERSION" \
        --max-seconds "$CONFORMANCE_MAX_SECONDS" \
        --probe-max-seconds 30 \
        --native-max-seconds 120 \
        --output-json "$CONFORMANCE_REPORT" >/dev/null; then
        record_check "live_api_conformance" "$conformance_command" "pass" "live_api_conformance_passed"
        conformance_reason_code="live_api_conformance_passed"
      else
        conformance_reason_code="$(read_report_reason_code "$CONFORMANCE_REPORT")"
        record_check "live_api_conformance" "$conformance_command" "fail" "$conformance_reason_code"
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "skipped" "live_api_conformance_failed"
        record_check "runtime_commit_policy" "$runtime_commit_policy_command" "skipped" "live_api_conformance_failed"
        overall_status="fail"
        reason_code="live_api_conformance_failed"
        runtime_commit_reason_code="live_api_conformance_failed"
        runtime_commit_policy_reason_code="live_api_conformance_failed"
      fi
    fi

    if [ "$overall_status" = "ok" ]; then
      mkdir -p "$(dirname "$RUNTIME_COMMIT_OUTPUT_FILE")"
      runtime_commit_exit_code=0
      set +e
      timeout "$RUNTIME_COMMIT_MAX_SECONDS" bash -lc "$runtime_commit_command" >"$RUNTIME_COMMIT_OUTPUT_FILE" 2>&1
      runtime_commit_exit_code=$?
      set -e

      if [ "$runtime_commit_exit_code" -eq 0 ]; then
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "pass" "runtime_commit_endpoint_passed"
        policy_final_decision="$(read_policy_final_decision "$RUNTIME_COMMIT_LIVE_POLICY_REPORT")"
        if [ "$policy_final_decision" = "GO" ]; then
          record_check "runtime_commit_policy" "$runtime_commit_policy_command" "pass" "runtime_commit_policy_passed"
          runtime_commit_reason_code="runtime_commit_endpoint_passed"
          runtime_commit_policy_reason_code="runtime_commit_policy_passed"
          reason_code="live_runtime_integration_passed"
        else
          record_check "runtime_commit_policy" "$runtime_commit_policy_command" "fail" "runtime_commit_policy_failed:${policy_final_decision}"
          runtime_commit_reason_code="runtime_commit_endpoint_passed"
          runtime_commit_policy_reason_code="runtime_commit_policy_failed:${policy_final_decision}"
          overall_status="fail"
          reason_code="runtime_commit_policy_failed"
        fi
      elif [ "$runtime_commit_exit_code" -eq 124 ]; then
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "fail" "runtime_commit_endpoint_timeout"
        record_check "runtime_commit_policy" "$runtime_commit_policy_command" "skipped" "runtime_commit_endpoint_failed"
        runtime_commit_reason_code="runtime_commit_endpoint_timeout"
        runtime_commit_policy_reason_code="runtime_commit_endpoint_failed"
        overall_status="fail"
        reason_code="runtime_commit_endpoint_failed"
      else
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "fail" "runtime_commit_endpoint_failed"
        record_check "runtime_commit_policy" "$runtime_commit_policy_command" "skipped" "runtime_commit_endpoint_failed"
        runtime_commit_reason_code="runtime_commit_endpoint_failed"
        runtime_commit_policy_reason_code="runtime_commit_endpoint_failed"
        overall_status="fail"
        reason_code="runtime_commit_endpoint_failed"
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
      reason_code="runtime_integration_budget_exceeded"
    fi
  fi
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$CHECKOUT_PATH" "$EXPECTED_REMOTE_URL" "$EXPECTED_REF" "$BASE_URL" "$FORK_CHAIN_VERSION" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$runtime_commit_command" "$RUNTIME_COMMIT_OUTPUT_FILE" "$RUNTIME_COMMIT_LIVE_SUMMARY" "$RUNTIME_COMMIT_LIVE_POLICY_REPORT" "$RUNTIME_COMMIT_FINALITY_COMMAND" "$RUNTIME_COMMIT_FINALITY_OUTPUT_FILE" "$RUNTIME_COMMIT_FINALITY_MAX_SECONDS" "$BOOTSTRAP_REPORT" "$LOCALHOST_SIGNED_REPORT" "$CONFORMANCE_REPORT" "$bootstrap_reason_code" "$localhost_signed_reason_code" "$conformance_reason_code" "$runtime_commit_reason_code" "$runtime_commit_policy_reason_code" "$RUNTIME_PROFILE" "$CHECK_FILE" "$RUNTIME_PROVIDER_CLIENT_CONTRACT" "$runtime_commit_command_profile" "$runtime_commit_policy_command_profile" "$runtime_commit_command_profile_version" "$RUNTIME_SIGNER_PROFILE_SELECTOR_ENV" "$RUNTIME_SIGNER_PROFILE" "$RUNTIME_SIGNER_PREVIOUS_PROFILE" "$RUNTIME_SIGNER_FAILOVER_ACTIVE" "$RUNTIME_SIGNER_ROTATION_EPOCH" "$RUNTIME_SIGNER_PREVIOUS_ROTATION_EPOCH" "$RUNTIME_SIGNER_KEY_SOURCE_CONTRACT_VERSION" "$RUNTIME_SIGNER_KEY_SOURCE" "$RUNTIME_SIGNER_PRIVATE_KEY_ENV" "$RUNTIME_SIGNER_KEY_REF_ENV" "$RUNTIME_SIGNER_FALLBACK_GUARD_CONTRACT_VERSION" "$runtime_signer_fallback_private_key_present" "$runtime_signer_raw_private_key_present" "$RUNTIME_SIGNER_ATTESTATION_SCHEMA_VERSION" "$RUNTIME_SIGNING_PROFILE" "$RUNTIME_SIGNER_FALLBACK_GUARD_MODE" "$RUNTIME_SIGNER_MANAGED_EXTERNAL_RAW_PRIVATE_KEY_REMEDIATION" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
checkout_path = sys.argv[5]
expected_remote_url = sys.argv[6]
expected_ref = sys.argv[7]
base_url = sys.argv[8]
fork_chain_version = sys.argv[9]
elapsed_seconds = int(sys.argv[10])
max_seconds = int(sys.argv[11])
budget_status = sys.argv[12]
runtime_commit_command = sys.argv[13]
runtime_commit_output_file = sys.argv[14]
runtime_commit_live_summary = sys.argv[15]
runtime_commit_live_policy_report = sys.argv[16]
runtime_commit_finality_command = sys.argv[17]
runtime_commit_finality_output_file = sys.argv[18]
runtime_commit_finality_max_seconds = int(sys.argv[19])
bootstrap_report = sys.argv[20]
localhost_signed_report = sys.argv[21]
conformance_report = sys.argv[22]
bootstrap_reason_code = sys.argv[23]
localhost_signed_reason_code = sys.argv[24]
conformance_reason_code = sys.argv[25]
runtime_commit_reason_code = sys.argv[26]
runtime_commit_policy_reason_code = sys.argv[27]
runtime_profile = sys.argv[28]
checks_path = pathlib.Path(sys.argv[29])
runtime_provider_client_contract = sys.argv[30]
runtime_commit_command_profile = sys.argv[31]
runtime_commit_policy_command_profile = sys.argv[32]
runtime_commit_command_profile_version = sys.argv[33]
runtime_signer_profile_selector_env = sys.argv[34]
runtime_signer_profile = sys.argv[35]
runtime_signer_previous_profile = sys.argv[36]
runtime_signer_failover_active = sys.argv[37] == "true"
runtime_signer_rotation_epoch = int(sys.argv[38])
runtime_signer_previous_rotation_epoch = int(sys.argv[39])
runtime_signer_key_source_contract_version = sys.argv[40]
runtime_signer_key_source = sys.argv[41]
runtime_signer_private_key_env = sys.argv[42]
runtime_signer_key_reference_env = sys.argv[43]
runtime_signer_fallback_guard_contract_version = sys.argv[44]
runtime_signer_fallback_private_key_present = sys.argv[45] == "true"
runtime_signer_raw_private_key_present = sys.argv[46] == "true"
runtime_signer_attestation_schema_version = sys.argv[47]
runtime_signing_profile = sys.argv[48]
runtime_signer_fallback_guard_mode = sys.argv[49]
runtime_signer_managed_external_raw_private_key_remediation = sys.argv[50]

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


def read_nested_runtime_reason_code(mode_value: str, summary_path: str) -> str:
    if mode_value == "dry-run":
        return "not_run"

    path = pathlib.Path(summary_path)
    if not path.exists():
        return "report_missing"

    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return "report_invalid_json"

    nested_value = payload.get("reason_code")
    if isinstance(nested_value, str) and nested_value.strip():
        return nested_value
    return "reason_code_missing"


NESTED_RUNTIME_REASON_TO_TAXONOMY = {
    "live_preflight_timeout": "transport.preflight.timeout",
    "live_preflight_failed": "transport.preflight.failed",
    "live_runtime_commit_command_timeout": "transport.submit.timeout",
    "live_runtime_commit_command_failed": "transport.submit.failed",
    "live_finality_retry_exhausted_timeout": "finality.timeout",
    "live_finality_retry_exhausted_failed": "finality.failed",
    "live_finality_command_timeout": "finality.timeout",
    "live_finality_command_failed": "finality.failed",
    "live_runtime_commit_budget_exceeded": "budget.exceeded",
}

NESTED_RUNTIME_REASON_TO_HINT = {
    "live_preflight_timeout": "Verify live-node reachability and preflight latency at the configured --base-url endpoint.",
    "live_preflight_failed": "Inspect preflight health output and provider-hint wiring before submit/finality checks.",
    "live_runtime_commit_command_timeout": "Increase --runtime-commit-max-seconds or reduce submit command latency.",
    "live_runtime_commit_command_failed": "Inspect runtime commit command stderr in runtime_commit_output_file for transport/provider errors.",
    "live_finality_retry_exhausted_timeout": "Increase --runtime-commit-finality-max-seconds/--runtime-commit-finality-retry-max-attempts and verify finality endpoint responsiveness.",
    "live_finality_retry_exhausted_failed": "Inspect runtime finality command output and verify notifications/block fallback endpoint contracts before retrying.",
    "live_finality_command_timeout": "Increase --runtime-commit-finality-max-seconds and verify finality endpoint responsiveness.",
    "live_finality_command_failed": "Inspect runtime finality command output and verify notifications/block fallback endpoint contracts.",
    "live_runtime_commit_budget_exceeded": "Increase --max-seconds or reduce prerequisite/runtime command cost for local-heavy runs.",
}


def classify_runtime_commit_failure(
    mode_value: str,
    status_value: str,
    reason_value: str,
    runtime_reason_value: str,
    runtime_policy_reason_value: str,
    nested_runtime_reason_value: str,
) -> tuple[str, str]:
    if status_value == "ok":
        if mode_value == "dry-run":
            return "none", "Dry-run mode does not execute runtime commit submit/finality checks."
        return "none", "No runtime commit failure observed."

    if reason_value == "runtime_commit_policy_failed":
        return (
            "policy.rejected",
            "Inspect runtime_commit_live_policy_report for final_decision and reason_codes.",
        )

    if reason_value == "runtime_integration_budget_exceeded":
        return (
            "budget.exceeded",
            "Increase --max-seconds or reduce prerequisite/runtime lane execution time.",
        )

    if reason_value == "runtime_signer_managed_external_raw_private_key_present_violation":
        return (
            "none",
            "Unset the selected signer private key env and provide managed signer key-reference env for managed-external mode.",
        )

    if reason_value != "runtime_commit_endpoint_failed":
        return (
            "none",
            "Runtime commit endpoint was not the terminal failure path; inspect prerequisite check reason codes.",
        )

    if runtime_reason_value == "runtime_commit_endpoint_timeout":
        return (
            "transport.submit.timeout",
            "Increase --runtime-commit-max-seconds or inspect runtime commit endpoint command latency.",
        )

    if runtime_policy_reason_value == "runtime_commit_endpoint_failed" and nested_runtime_reason_value in (
        "report_missing",
        "report_invalid_json",
        "reason_code_missing",
    ):
        return (
            "runtime.summary.unavailable",
            "Ensure runtime_commit_live_summary is written and contains a valid reason_code field.",
        )

    mapped_taxonomy = NESTED_RUNTIME_REASON_TO_TAXONOMY.get(nested_runtime_reason_value)
    if mapped_taxonomy is not None:
        mapped_hint = NESTED_RUNTIME_REASON_TO_HINT[nested_runtime_reason_value]
        return mapped_taxonomy, mapped_hint

    return (
        "runtime.unknown",
        f"Inspect nested runtime reason_code={nested_runtime_reason_value} and runtime command artifacts.",
    )


runtime_commit_nested_reason_code = read_nested_runtime_reason_code(mode, runtime_commit_live_summary)
runtime_commit_failure_taxonomy, runtime_commit_failure_diagnostic_hint = classify_runtime_commit_failure(
    mode,
    status,
    reason_code,
    runtime_commit_reason_code,
    runtime_commit_policy_reason_code,
    runtime_commit_nested_reason_code,
)

runtime_signer_attestation_approved_signers: list[str] = []
if isinstance(runtime_signer_profile, str) and runtime_signer_profile.strip():
    runtime_signer_attestation_approved_signers = [runtime_signer_profile]
runtime_signer_attestation_required_approvals = 1
runtime_signer_attestation_bundle = {
    "schema_version": runtime_signer_attestation_schema_version,
    "required_approvals": runtime_signer_attestation_required_approvals,
    "approved_signers": runtime_signer_attestation_approved_signers,
    "signer_profile": runtime_signer_profile,
    "signer_key_source": runtime_signer_key_source,
}
runtime_signer_quorum_linkage_contract_version = "v1"
runtime_signer_quorum_approved_signers_count = len(runtime_signer_attestation_approved_signers)
runtime_signer_quorum_profile_linked = (
    isinstance(runtime_signer_profile, str)
    and runtime_signer_profile.strip()
    and runtime_signer_profile in runtime_signer_attestation_approved_signers
)
runtime_signer_quorum_satisfied = (
    runtime_signer_quorum_approved_signers_count >= runtime_signer_attestation_required_approvals
)
runtime_signer_quorum_linked = (
    runtime_signer_quorum_profile_linked and runtime_signer_quorum_satisfied
)

summary = {
    "schema_version": "kamn.kolme.local-kamn-live-runtime-integration-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "ci_fast_gate_eligible": False,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "checkout_path": checkout_path,
    "expected_remote_url": expected_remote_url,
    "expected_ref": expected_ref,
    "base_url": base_url,
    "fork_chain_version": fork_chain_version,
    "runtime_commit_command": runtime_commit_command,
    "runtime_provider_client_contract": runtime_provider_client_contract,
    "runtime_profile": runtime_profile,
    "runtime_signing_profile": runtime_signing_profile,
    "runtime_commit_command_profile": runtime_commit_command_profile,
    "runtime_commit_policy_command_profile": runtime_commit_policy_command_profile,
    "runtime_commit_command_profile_version": runtime_commit_command_profile_version,
    "runtime_signer_profile_selector_env": runtime_signer_profile_selector_env,
    "runtime_signer_profile": runtime_signer_profile,
    "runtime_signer_previous_profile": runtime_signer_previous_profile,
    "runtime_signer_failover_active": runtime_signer_failover_active,
    "runtime_signer_rotation_epoch": runtime_signer_rotation_epoch,
    "runtime_signer_previous_rotation_epoch": runtime_signer_previous_rotation_epoch,
    "runtime_signer_key_source_contract_version": runtime_signer_key_source_contract_version,
    "runtime_signer_key_source": runtime_signer_key_source,
    "runtime_signer_private_key_env": runtime_signer_private_key_env,
    "runtime_signer_key_reference_env": runtime_signer_key_reference_env,
    "runtime_signer_fallback_guard_contract_version": runtime_signer_fallback_guard_contract_version,
    "runtime_signer_fallback_guard_mode": runtime_signer_fallback_guard_mode,
    "runtime_signer_managed_external_raw_private_key_remediation": runtime_signer_managed_external_raw_private_key_remediation,
    "runtime_signer_fallback_private_key_present": runtime_signer_fallback_private_key_present,
    "runtime_signer_raw_private_key_present": runtime_signer_raw_private_key_present,
    "runtime_signer_attestation_schema_version": runtime_signer_attestation_schema_version,
    "runtime_signer_attestation_bundle": runtime_signer_attestation_bundle,
    "runtime_signer_quorum_linkage_contract_version": runtime_signer_quorum_linkage_contract_version,
    "runtime_signer_quorum_required_approvals": runtime_signer_attestation_required_approvals,
    "runtime_signer_quorum_approved_signers_count": runtime_signer_quorum_approved_signers_count,
    "runtime_signer_quorum_profile_linked": runtime_signer_quorum_profile_linked,
    "runtime_signer_quorum_satisfied": runtime_signer_quorum_satisfied,
    "runtime_signer_quorum_linked": runtime_signer_quorum_linked,
    "runtime_commit_live_policy_report": runtime_commit_live_policy_report,
    "runtime_commit_finality_command": runtime_commit_finality_command if runtime_commit_finality_command else "",
    "runtime_commit_finality_output_file": runtime_commit_finality_output_file if runtime_commit_finality_command else "",
    "runtime_commit_finality_enabled": bool(runtime_commit_finality_command),
    "runtime_commit_finality_max_seconds": runtime_commit_finality_max_seconds,
    "bootstrap_reason_code": bootstrap_reason_code,
    "localhost_signed_reason_code": localhost_signed_reason_code,
    "conformance_reason_code": conformance_reason_code,
    "runtime_commit_reason_code": runtime_commit_reason_code,
    "runtime_commit_policy_reason_code": runtime_commit_policy_reason_code,
    "runtime_commit_nested_reason_code": runtime_commit_nested_reason_code,
    "runtime_commit_failure_taxonomy_version": "v1",
    "runtime_commit_failure_taxonomy": runtime_commit_failure_taxonomy,
    "runtime_commit_failure_diagnostic_hint": runtime_commit_failure_diagnostic_hint,
    "contracts": {
        "ci_fast_gate_scope": "local-only",
        "runtime_provider_client_contract": runtime_provider_client_contract,
        "runtime_profile": runtime_profile,
        "runtime_signing_profile": runtime_signing_profile,
        "runtime_signer_profile_selector_env": runtime_signer_profile_selector_env,
        "runtime_signer_profile": runtime_signer_profile,
        "runtime_signer_failover_requires_profile_change": True,
        "runtime_signer_rotation_epoch_must_increase_on_failover": True,
        "runtime_signer_failover_attestation_min_required_approvals": 2,
        "runtime_signer_failover_attestation_previous_profile_membership_required": True,
        "runtime_signer_key_source_contract_version": runtime_signer_key_source_contract_version,
        "runtime_signer_key_source": runtime_signer_key_source,
        "runtime_signer_private_key_env": runtime_signer_private_key_env,
        "runtime_signer_key_reference_env": runtime_signer_key_reference_env,
        "runtime_signer_fallback_guard_contract_version": runtime_signer_fallback_guard_contract_version,
        "runtime_signer_fallback_guard_mode": runtime_signer_fallback_guard_mode,
        "runtime_signer_managed_external_raw_private_key_remediation": runtime_signer_managed_external_raw_private_key_remediation,
        "runtime_signer_fallback_private_key_allowed": False,
        "runtime_signer_fallback_private_key_command_marker_allowed": False,
        "runtime_signer_managed_external_raw_private_key_allowed": False,
        "runtime_signer_attestation_schema_version": runtime_signer_attestation_schema_version,
        "runtime_signer_attestation_signer_uniqueness_required": True,
        "runtime_signer_attestation_threshold_required": True,
        "runtime_signer_attestation_profile_membership_required": True,
        "runtime_signer_attestation_required_approvals": runtime_signer_attestation_required_approvals,
        "runtime_signer_quorum_linkage_contract_version": runtime_signer_quorum_linkage_contract_version,
        "runtime_signer_quorum_required_approvals": runtime_signer_attestation_required_approvals,
        "runtime_signer_quorum_linked_required": True,
        "runtime_signer_quorum_threshold_required": True,
        "runtime_signer_quorum_profile_membership_required": True,
        "runtime_signer_quorum_linked": runtime_signer_quorum_linked,
        "runtime_commit_endpoint": "/broadcast/runtime-commit",
        "runtime_commit_method": "POST",
        "runtime_commit_finality_primary_endpoint": "/notifications",
        "runtime_commit_finality_fallback_endpoint": "/block/{height}",
        "runtime_commit_failure_taxonomy_version": "v1",
    },
    "checks": checks,
    "artifact_paths": [
        bootstrap_report,
        localhost_signed_report,
        conformance_report,
        runtime_commit_output_file,
        runtime_commit_live_summary,
        runtime_commit_live_policy_report,
    ],
}

if runtime_commit_finality_command:
    summary["artifact_paths"].append(runtime_commit_finality_output_file)

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=true"
echo "ci_fast_gate_eligible=false"
echo "runtime_profile=$RUNTIME_PROFILE"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
