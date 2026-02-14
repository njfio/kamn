#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SOURCE_FILE="$ROOT_DIR/crates/kamn-node/src/service_api_endpoint.rs"
TEST_FILE="$ROOT_DIR/crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs"

output_json=""
max_seconds=240

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi

if [ ! -f "$SOURCE_FILE" ]; then
  echo "expected service api endpoint source file: $SOURCE_FILE" >&2
  exit 1
fi
if [ ! -f "$TEST_FILE" ]; then
  echo "expected service api endpoint test file: $TEST_FILE" >&2
  exit 1
fi

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

for source_marker in \
  "use serde::de::DeserializeOwned;" \
  "use serde::{Deserialize, Serialize};" \
  "fn parse_service_api_payload<T: DeserializeOwned>" \
  "fn service_api_payload_decode_reason_code(" \
  "fn serialize_service_api_json<T: Serialize>(" \
  "serde_json::from_str(payload)" \
  "serde_json::to_string(payload)"; do
  if ! grep -Fq "$source_marker" "$SOURCE_FILE"; then
    echo "missing required serde source marker: $source_marker" >&2
    exit 1
  fi
done

for dto_marker in \
  "ServiceApiErrorBody" \
  "ServiceApiHealthBody" \
  "ServiceApiMessageCreateBody" \
  "ServiceApiMessageGetBody" \
  "ServiceApiChannelCreateBody" \
  "ServiceApiChannelMessagesBody" \
  "ServiceApiTaskCreateBody" \
  "ServiceApiTaskGetBody" \
  "ServiceApiAgentGetBody" \
  "ServiceApiWebsocketStateTransitionBody"; do
  if ! grep -Fq "$dto_marker" "$SOURCE_FILE"; then
    echo "missing required serde dto marker: $dto_marker" >&2
    exit 1
  fi
done

for decode_reason in \
  "service_api_payload_json_syntax_invalid" \
  "service_api_payload_structure_invalid" \
  "service_api_payload_io_error"; do
  if ! grep -Fq "$decode_reason" "$SOURCE_FILE"; then
    echo "missing required serde decode reason marker: $decode_reason" >&2
    exit 1
  fi
done

for test_marker in \
  "unit_service_api_endpoint_serde_payload_roundtrip_contracts" \
  "integration_service_api_endpoint_http_response_bodies_match_serde_contracts" \
  "regression_service_api_payload_parse_reason_codes_fail_closed"; do
  if ! grep -Fq "$test_marker" "$TEST_FILE"; then
    echo "missing required serde test marker: $test_marker" >&2
    exit 1
  fi
done

pushd "$ROOT_DIR" >/dev/null
cargo test -p kamn-node main_tests::unit_service_api_endpoint_serde_payload_roundtrip_contracts -- --exact \
  >"$TMP_DIR/service-api-serde-unit.log" 2>&1
cargo test -p kamn-node main_tests::integration_service_api_endpoint_http_response_bodies_match_serde_contracts -- --exact \
  >"$TMP_DIR/service-api-serde-integration.log" 2>&1
cargo test -p kamn-node main_tests::regression_service_api_payload_parse_reason_codes_fail_closed -- --exact \
  >"$TMP_DIR/service-api-serde-regression.log" 2>&1
popd >/dev/null

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "service api serde payload parity live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

summary_report="$TMP_DIR/service-api-serde-payload-parity-live-summary.json"
python3 - "$summary_report" "$elapsed_seconds" "$max_seconds" <<'PY'
import json
import pathlib
import sys

summary_report = pathlib.Path(sys.argv[1])
elapsed_seconds = int(sys.argv[2])
max_seconds = int(sys.argv[3])

payload = {
    "schema_version": "kamn.runtime.service-api-serde-payload-parity-live-validation.v1",
    "status": "pass",
    "final_decision": "GO",
    "serde_dto_status": "verified",
    "serde_serializer_status": "verified",
    "serde_parser_status": "verified",
    "route_payload_parity_status": "verified",
    "fail_closed_status": "verified",
    "performance_budget_status": "verified",
    "fail_closed_reason_code": "service_api_payload_structure_invalid",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
}
summary_report.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

if [[ -n "$output_json" ]]; then
  cp "$summary_report" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "serde_dto_status=verified"
echo "serde_serializer_status=verified"
echo "serde_parser_status=verified"
echo "route_payload_parity_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=service_api_payload_structure_invalid"
echo "performance_budget_status=verified"
