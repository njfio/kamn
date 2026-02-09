#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh \
    --output-file <path> \
    --escrow-id <value> \
    --settlement-outcome RELEASED|REFUNDED|TIMEOUT_REFUNDED|DISPUTED_RESOLVED \
    --receipt-id <value> \
    --receipt-finality FINAL|PENDING|FAILED \
    --expected-release-amount <n> \
    --expected-refund-amount <n> \
    --observed-release-amount <n> \
    --observed-refund-amount <n> \
    --ledger-reference-id <value> \
    --timeout-elapsed true|false \
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
escrow_id=""
settlement_outcome=""
receipt_id=""
receipt_finality=""
expected_release_amount=""
expected_refund_amount=""
observed_release_amount=""
observed_refund_amount=""
ledger_reference_id=""
ledger_reference_id_set=false
timeout_elapsed=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --escrow-id)
      escrow_id="${2:-}"
      shift 2
      ;;
    --settlement-outcome)
      settlement_outcome="${2:-}"
      shift 2
      ;;
    --receipt-id)
      receipt_id="${2:-}"
      shift 2
      ;;
    --receipt-finality)
      receipt_finality="${2:-}"
      shift 2
      ;;
    --expected-release-amount)
      expected_release_amount="${2:-}"
      shift 2
      ;;
    --expected-refund-amount)
      expected_refund_amount="${2:-}"
      shift 2
      ;;
    --observed-release-amount)
      observed_release_amount="${2:-}"
      shift 2
      ;;
    --observed-refund-amount)
      observed_refund_amount="${2:-}"
      shift 2
      ;;
    --ledger-reference-id)
      ledger_reference_id="${2-}"
      ledger_reference_id_set=true
      shift 2
      ;;
    --timeout-elapsed)
      timeout_elapsed="${2:-}"
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

if [[ -z "$output_file" || -z "$escrow_id" || -z "$settlement_outcome" || -z "$receipt_id" || -z "$receipt_finality" || -z "$expected_release_amount" || -z "$expected_refund_amount" || -z "$observed_release_amount" || -z "$observed_refund_amount" || "$ledger_reference_id_set" != true || -z "$timeout_elapsed" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all settlement evidence bundle arguments are required"
fi

require_int "expected-release-amount" "$expected_release_amount"
require_int "expected-refund-amount" "$expected_refund_amount"
require_int "observed-release-amount" "$observed_release_amount"
require_int "observed-refund-amount" "$observed_refund_amount"

if [[ "$timeout_elapsed" != "true" && "$timeout_elapsed" != "false" ]]; then
  fail "timeout-elapsed must be true or false"
fi

mkdir -p "$(dirname "$output_file")"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$escrow_id" "$settlement_outcome" "$receipt_id" "$receipt_finality" "$expected_release_amount" "$expected_refund_amount" "$observed_release_amount" "$observed_refund_amount" "$ledger_reference_id" "$timeout_elapsed" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    escrow_id,
    settlement_outcome,
    receipt_id,
    receipt_finality,
    expected_release_raw,
    expected_refund_raw,
    observed_release_raw,
    observed_refund_raw,
    ledger_reference_id,
    timeout_elapsed_raw,
    ci_fast_gate,
) = sys.argv[1:]

if settlement_outcome not in {"RELEASED", "REFUNDED", "TIMEOUT_REFUNDED", "DISPUTED_RESOLVED"}:
    fail("settlement-outcome must be RELEASED|REFUNDED|TIMEOUT_REFUNDED|DISPUTED_RESOLVED")

if receipt_finality not in {"FINAL", "PENDING", "FAILED"}:
    fail("receipt-finality must be FINAL|PENDING|FAILED")

if timeout_elapsed_raw not in {"true", "false"}:
    fail("timeout-elapsed must be true or false")
timeout_elapsed = timeout_elapsed_raw == "true"

if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci-fast-gate must be PASS or FAIL")

expected_release = int(expected_release_raw)
expected_refund = int(expected_refund_raw)
observed_release = int(observed_release_raw)
observed_refund = int(observed_refund_raw)

decision_reasons: list[str] = []

if not receipt_id.strip() or receipt_finality != "FINAL":
    decision_reasons.append("missing or invalid receipt evidence")

if expected_release != observed_release or expected_refund != observed_refund:
    decision_reasons.append("ledger amount drift detected")

if not ledger_reference_id.strip():
    decision_reasons.append("missing ledger reference evidence")

if settlement_outcome == "TIMEOUT_REFUNDED" and not timeout_elapsed:
    decision_reasons.append("timeout refund without elapsed timeout")

if ci_fast_gate != "PASS":
    decision_reasons.append("ci-fast-gate-failed")

final_decision = "GO" if not decision_reasons else "NO-GO"
if not decision_reasons:
    decision_reasons.append("all settlement reconciliation gates satisfied")

payload = {
    "schema_version": "kamn.escrow.settlement-reconciliation.v1",
    "generated_at": generated_at,
    "escrow_id": escrow_id,
    "settlement_outcome": settlement_outcome,
    "receipt": {
        "receipt_id": receipt_id,
        "finality": receipt_finality,
    },
    "expected_amounts": {
        "release": expected_release,
        "refund": expected_refund,
    },
    "observed_amounts": {
        "release": observed_release,
        "refund": observed_refund,
    },
    "ledger": {
        "reference_id": ledger_reference_id,
    },
    "timeout_elapsed": timeout_elapsed,
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
