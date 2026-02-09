#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/token/generate_token_launch_handoff_evidence_bundle.sh \
    --output-file <path> \
    --token-symbol <value> \
    --configured-total-supply <n> \
    --expected-total-supply <n> \
    --configured-allocation-sum <n> \
    --expected-allocation-sum <n> \
    --allocation-bucket-count <n> \
    --expected-bucket-count <n> \
    --genesis-hash <sha256:...> \
    --required-approvals <n> \
    --received-approvals <n> \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

require_int() {
  local name="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "${name} must be an integer"
  fi
}

output_file=""
token_symbol=""
configured_total_supply=""
expected_total_supply=""
configured_allocation_sum=""
expected_allocation_sum=""
allocation_bucket_count=""
expected_bucket_count=""
genesis_hash=""
required_approvals=""
received_approvals=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --token-symbol)
      token_symbol="${2:-}"
      shift 2
      ;;
    --configured-total-supply)
      configured_total_supply="${2:-}"
      shift 2
      ;;
    --expected-total-supply)
      expected_total_supply="${2:-}"
      shift 2
      ;;
    --configured-allocation-sum)
      configured_allocation_sum="${2:-}"
      shift 2
      ;;
    --expected-allocation-sum)
      expected_allocation_sum="${2:-}"
      shift 2
      ;;
    --allocation-bucket-count)
      allocation_bucket_count="${2:-}"
      shift 2
      ;;
    --expected-bucket-count)
      expected_bucket_count="${2:-}"
      shift 2
      ;;
    --genesis-hash)
      genesis_hash="${2:-}"
      shift 2
      ;;
    --required-approvals)
      required_approvals="${2:-}"
      shift 2
      ;;
    --received-approvals)
      received_approvals="${2:-}"
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

if [[ -z "$output_file" || -z "$token_symbol" || -z "$configured_total_supply" || -z "$expected_total_supply" || -z "$configured_allocation_sum" || -z "$expected_allocation_sum" || -z "$allocation_bucket_count" || -z "$expected_bucket_count" || -z "$genesis_hash" || -z "$required_approvals" || -z "$received_approvals" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all token launch handoff evidence bundle arguments are required"
fi

require_int "configured-total-supply" "$configured_total_supply"
require_int "expected-total-supply" "$expected_total_supply"
require_int "configured-allocation-sum" "$configured_allocation_sum"
require_int "expected-allocation-sum" "$expected_allocation_sum"
require_int "allocation-bucket-count" "$allocation_bucket_count"
require_int "expected-bucket-count" "$expected_bucket_count"
require_int "required-approvals" "$required_approvals"
require_int "received-approvals" "$received_approvals"

mkdir -p "$(dirname "$output_file")"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$token_symbol" "$configured_total_supply" "$expected_total_supply" "$configured_allocation_sum" "$expected_allocation_sum" "$allocation_bucket_count" "$expected_bucket_count" "$genesis_hash" "$required_approvals" "$received_approvals" "$ci_fast_gate" <<'PY'
import json
import pathlib
import re
import sys


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    token_symbol,
    configured_total_raw,
    expected_total_raw,
    configured_allocation_raw,
    expected_allocation_raw,
    allocation_bucket_count_raw,
    expected_bucket_count_raw,
    genesis_hash,
    required_approvals_raw,
    received_approvals_raw,
    ci_fast_gate,
) = sys.argv[1:]

if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci-fast-gate must be PASS or FAIL")

if not re.fullmatch(r"[A-Z0-9]+", token_symbol or ""):
    fail("token-symbol must be uppercase alphanumeric")

configured_total = int(configured_total_raw)
expected_total = int(expected_total_raw)
configured_allocation = int(configured_allocation_raw)
expected_allocation = int(expected_allocation_raw)
allocation_bucket_count = int(allocation_bucket_count_raw)
expected_bucket_count = int(expected_bucket_count_raw)
required_approvals = int(required_approvals_raw)
received_approvals = int(received_approvals_raw)

decision_reasons: list[str] = []

if configured_total != expected_total:
    decision_reasons.append("configured total supply does not match expected total supply")

if configured_allocation != expected_allocation:
    decision_reasons.append("configured allocation sum does not match expected allocation sum")

if configured_allocation != configured_total:
    decision_reasons.append("configured allocation sum does not match configured total supply")

if expected_allocation != expected_total:
    decision_reasons.append("expected allocation sum does not match expected total supply")

if allocation_bucket_count != expected_bucket_count or allocation_bucket_count <= 0:
    decision_reasons.append("allocation bucket count mismatch")

if required_approvals <= 0:
    decision_reasons.append("required approvals must be greater than zero")

if received_approvals < required_approvals:
    decision_reasons.append("received approvals are below required approvals")

if not genesis_hash.startswith("sha256:") or len(genesis_hash) <= len("sha256:"):
    decision_reasons.append("genesis hash must be a non-empty sha256 digest")

if ci_fast_gate != "PASS":
    decision_reasons.append("ci-fast-gate-failed")

final_decision = "GO" if not decision_reasons else "NO-GO"
if not decision_reasons:
    decision_reasons.append("all token launch handoff invariants satisfied")

payload = {
    "schema_version": "kamn.token.launch-handoff.v1",
    "generated_at": generated_at,
    "token_symbol": token_symbol,
    "supply": {
        "configured_total_supply": configured_total,
        "expected_total_supply": expected_total,
    },
    "allocations": {
        "configured_sum": configured_allocation,
        "expected_sum": expected_allocation,
        "bucket_count": allocation_bucket_count,
        "expected_bucket_count": expected_bucket_count,
    },
    "genesis": {
        "genesis_hash": genesis_hash,
    },
    "approvals": {
        "required": required_approvals,
        "received": received_approvals,
    },
    "ci_fast_gate": ci_fast_gate,
    "decision_reasons": decision_reasons,
    "final_decision": final_decision,
}

path = pathlib.Path(output_file)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
print(final_decision)
PY
)"

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'final_decision=%s\n' "$final_decision"
