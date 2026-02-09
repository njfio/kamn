#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/bridge/generate_bridge_adapter_conformance_evidence_bundle.sh \
    --output-file <path> \
    --adapter-id <value> \
    --bridge-network ethereum|solana|near|custom \
    --dry-run true|false \
    --request-expected-schema-version <value> \
    --request-observed-schema-version <value> \
    --request-required-fields <csv> \
    --request-observed-fields <csv> \
    --receipt-expected-schema-version <value> \
    --receipt-observed-schema-version <value> \
    --receipt-required-fields <csv> \
    --receipt-observed-fields <csv> \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

output_file=""
adapter_id=""
bridge_network=""
dry_run=""
request_expected_schema_version=""
request_observed_schema_version=""
request_required_fields=""
request_observed_fields=""
receipt_expected_schema_version=""
receipt_observed_schema_version=""
receipt_required_fields=""
receipt_observed_fields=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --adapter-id)
      adapter_id="${2:-}"
      shift 2
      ;;
    --bridge-network)
      bridge_network="${2:-}"
      shift 2
      ;;
    --dry-run)
      dry_run="${2:-}"
      shift 2
      ;;
    --request-expected-schema-version)
      request_expected_schema_version="${2:-}"
      shift 2
      ;;
    --request-observed-schema-version)
      request_observed_schema_version="${2:-}"
      shift 2
      ;;
    --request-required-fields)
      request_required_fields="${2:-}"
      shift 2
      ;;
    --request-observed-fields)
      request_observed_fields="${2:-}"
      shift 2
      ;;
    --receipt-expected-schema-version)
      receipt_expected_schema_version="${2:-}"
      shift 2
      ;;
    --receipt-observed-schema-version)
      receipt_observed_schema_version="${2:-}"
      shift 2
      ;;
    --receipt-required-fields)
      receipt_required_fields="${2:-}"
      shift 2
      ;;
    --receipt-observed-fields)
      receipt_observed_fields="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [[ -z "$output_file" || -z "$adapter_id" || -z "$bridge_network" || -z "$dry_run" || -z "$request_expected_schema_version" || -z "$request_observed_schema_version" || -z "$request_required_fields" || -z "$request_observed_fields" || -z "$receipt_expected_schema_version" || -z "$receipt_observed_schema_version" || -z "$receipt_required_fields" || -z "$receipt_observed_fields" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all bridge adapter conformance evidence arguments are required"
fi

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$adapter_id" "$bridge_network" "$dry_run" "$request_expected_schema_version" "$request_observed_schema_version" "$request_required_fields" "$request_observed_fields" "$receipt_expected_schema_version" "$receipt_observed_schema_version" "$receipt_required_fields" "$receipt_observed_fields" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    raise ValueError(message)


def parse_csv(raw: str) -> list[str]:
    fields = [part.strip() for part in raw.split(",")]
    normalized = sorted({field for field in fields if field})
    return normalized


(
    output_file,
    generated_at,
    adapter_id,
    bridge_network,
    dry_run_raw,
    request_expected_schema_version,
    request_observed_schema_version,
    request_required_fields_raw,
    request_observed_fields_raw,
    receipt_expected_schema_version,
    receipt_observed_schema_version,
    receipt_required_fields_raw,
    receipt_observed_fields_raw,
    ci_fast_gate,
) = sys.argv[1:]

if bridge_network not in {"ethereum", "solana", "near", "custom"}:
    fail("bridge-network must be ethereum|solana|near|custom")

if dry_run_raw not in {"true", "false"}:
    fail("dry-run must be true or false")
dry_run = dry_run_raw == "true"

if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci-fast-gate must be PASS or FAIL")

if not adapter_id.strip():
    fail("adapter-id must be non-empty")

request_required_fields = parse_csv(request_required_fields_raw)
request_observed_fields = parse_csv(request_observed_fields_raw)
receipt_required_fields = parse_csv(receipt_required_fields_raw)
receipt_observed_fields = parse_csv(receipt_observed_fields_raw)

request_missing_required_fields = sorted(
    [field for field in request_required_fields if field not in set(request_observed_fields)]
)
receipt_missing_required_fields = sorted(
    [field for field in receipt_required_fields if field not in set(receipt_observed_fields)]
)

reason_codes: list[str] = []

if not dry_run:
    reason_codes.append("dry_run_disabled")
if request_expected_schema_version != request_observed_schema_version:
    reason_codes.append("request_schema_version_mismatch")
if receipt_expected_schema_version != receipt_observed_schema_version:
    reason_codes.append("receipt_schema_version_mismatch")
if not request_required_fields:
    reason_codes.append("request_required_fields_contract_missing")
if request_missing_required_fields:
    reason_codes.append("request_required_fields_missing")
if not receipt_required_fields:
    reason_codes.append("receipt_required_fields_contract_missing")
if receipt_missing_required_fields:
    reason_codes.append("receipt_required_fields_missing")
if ci_fast_gate != "PASS":
    reason_codes.append("ci_fast_gate_failed")

final_decision = "GO" if not reason_codes else "NO-GO"
if final_decision == "GO":
    reason_codes = [
        "adapter_conformance_dry_run_mode",
        "adapter_request_receipt_contracts_compatible",
    ]

reason_key = f"bridge_adapter_conformance_reason_codes:{final_decision}:v1"

payload = {
    "schema_version": "kamn.bridge.adapter-conformance.v1",
    "generated_at": generated_at,
    "adapter_id": adapter_id,
    "bridge_network": bridge_network,
    "dry_run": dry_run,
    "request_contract": {
        "expected_schema_version": request_expected_schema_version,
        "observed_schema_version": request_observed_schema_version,
        "required_fields": request_required_fields,
        "observed_fields": request_observed_fields,
        "missing_required_fields": request_missing_required_fields,
    },
    "receipt_contract": {
        "expected_schema_version": receipt_expected_schema_version,
        "observed_schema_version": receipt_observed_schema_version,
        "required_fields": receipt_required_fields,
        "observed_fields": receipt_observed_fields,
        "missing_required_fields": receipt_missing_required_fields,
    },
    "ci_fast_gate": ci_fast_gate,
    "reason_key": reason_key,
    "reason_codes": reason_codes,
    "decision_reasons": reason_codes,
    "final_decision": final_decision,
}

path = pathlib.Path(output_file)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
print(final_decision)
PY
)"

reason_key="bridge_adapter_conformance_reason_codes:${final_decision}:v1"

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'final_decision=%s\n' "$final_decision"
printf 'reason_key=%s\n' "$reason_key"
