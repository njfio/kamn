#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: run_critical_path_mutation_gate.sh [options]

Runs a bounded mutation gate across critical runtime/API/network/security paths.

Options:
  --output-json <path>       Output mutation report JSON.
  --timeout-seconds <secs>   Per-slice cargo mutants timeout (default: 900).

Environment:
  KAMN_MUTATION_GATE_STUB=true
      Emit deterministic stub outputs for script regression tests instead of
      executing cargo mutants.
USAGE
}

output_json="ci-critical-path-mutation-report.json"
timeout_seconds="900"

while (($# > 0)); do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --timeout-seconds)
      timeout_seconds="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if ! [[ "$timeout_seconds" =~ ^[0-9]+$ ]] || [ "$timeout_seconds" -le 0 ]; then
  echo "timeout-seconds must be a positive integer" >&2
  exit 2
fi

mutation_gate_stub="${KAMN_MUTATION_GATE_STUB:-false}"
if [ "$mutation_gate_stub" != "true" ]; then
  if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo is required" >&2
    exit 2
  fi
  if ! cargo mutants --version >/dev/null 2>&1; then
    echo "cargo mutants is required; install via cargo install cargo-mutants --locked" >&2
    exit 2
  fi
fi

tmp_dir="$(mktemp -d)"
slice_tsv="$tmp_dir/slices.tsv"
trap 'rm -rf "$tmp_dir"' EXIT

parse_summary_line() {
  local summary_line="$1"
  python3 - "$summary_line" <<'PY'
import re
import sys

line = sys.argv[1]
line = line.strip()
if not line:
    print("parse_error=1")
    print("tested=0")
    print("caught=0")
    print("missed=0")
    print("unviable=0")
    print("timeout=0")
    raise SystemExit(0)

match = re.search(r"(\d+)\s+mutants? tested in .*:\s*(.+)$", line)
if not match:
    print("parse_error=1")
    print("tested=0")
    print("caught=0")
    print("missed=0")
    print("unviable=0")
    print("timeout=0")
    raise SystemExit(0)

tested = int(match.group(1))
counts = {"caught": 0, "missed": 0, "unviable": 0, "timeout": 0}
for chunk in match.group(2).split(","):
    part = chunk.strip()
    item = re.match(r"(\d+)\s+([a-z_]+)", part)
    if not item:
        continue
    value = int(item.group(1))
    key = item.group(2)
    if key in counts:
        counts[key] = value

print("parse_error=0")
print(f"tested={tested}")
for key in ("caught", "missed", "unviable", "timeout"):
    print(f"{key}={counts[key]}")
PY
}

run_slice() {
  local slice_id="$1"
  local expected_mutants="$2"
  shift 2

  local log_file="$tmp_dir/${slice_id}.log"
  local exit_code=0
  if [ "${KAMN_MUTATION_GATE_STUB:-false}" = "true" ]; then
    if [ "${KAMN_MUTATION_GATE_STUB_FAIL_SLICE:-}" = "$slice_id" ]; then
      {
        echo "Found ${expected_mutants} mutants to test"
        echo "ok       Unmutated baseline in 0s build + 0s test"
        echo "${expected_mutants} mutants tested in 0s: 1 missed, $(( expected_mutants - 1 )) caught"
      } >"$log_file"
      exit_code=2
    else
      {
        echo "Found ${expected_mutants} mutants to test"
        echo "ok       Unmutated baseline in 0s build + 0s test"
        echo "${expected_mutants} mutants tested in 0s: ${expected_mutants} caught"
      } >"$log_file"
    fi
  else
    set +e
    "$@" >"$log_file" 2>&1
    exit_code=$?
    set -e
  fi

  local found_mutants
  found_mutants="$(grep -Eo 'Found [0-9]+ mutants? to test' "$log_file" | tail -n 1 | awk '{print $2}' || true)"
  local summary_line
  summary_line="$(grep -E '[0-9]+ mutants? tested in ' "$log_file" | tail -n 1 || true)"

  local parse_error tested caught missed unviable timeout
  parse_error=1
  tested=0
  caught=0
  missed=0
  unviable=0
  timeout=0
  eval "$(parse_summary_line "$summary_line")"

  local status="ok"
  local reasons=()
  if [ -z "$found_mutants" ]; then
    reasons+=("critical_path_mutation_slice_discovery_missing")
    status="fail"
    found_mutants=0
  elif [ "$found_mutants" -ne "$expected_mutants" ]; then
    reasons+=("critical_path_mutation_slice_expected_count_mismatch")
    status="fail"
  fi

  if [ "$parse_error" -ne 0 ]; then
    reasons+=("critical_path_mutation_slice_summary_parse_failed")
    status="fail"
  fi

  if [ "$exit_code" -ne 0 ]; then
    reasons+=("critical_path_mutation_slice_exit_nonzero")
    status="fail"
  fi

  if [ "$missed" -gt 0 ] || [ "$unviable" -gt 0 ] || [ "$timeout" -gt 0 ]; then
    reasons+=("critical_path_mutation_slice_escape_detected")
    status="fail"
  fi

  local reasons_csv="none"
  if [ "${#reasons[@]}" -gt 0 ]; then
    reasons_csv="$(IFS=,; echo "${reasons[*]}")"
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$slice_id" \
    "$expected_mutants" \
    "$found_mutants" \
    "$tested" \
    "$caught" \
    "$missed" \
    "$unviable" \
    "$timeout" \
    "$exit_code" \
    "$status" \
    "$reasons_csv" \
    >>"$slice_tsv"
}

run_slice "core-direct-message-crypto" 2 \
  cargo mutants -p kamn-core \
    --file crates/kamn-core/src/direct_message_crypto.rs \
    --re "direct_message_crypto\\.rs:(73:18|115:13):" \
    --copy-vcs true \
    --cargo-test-arg --lib \
    --cargo-test-arg direct_message_crypto::tests::decrypt_rejects_algorithm_mismatch \
    --timeout "$timeout_seconds"

run_slice "core-group-channel-crypto" 1 \
  cargo mutants -p kamn-core \
    --file crates/kamn-core/src/group_channel_crypto.rs \
    --re "group_channel_crypto\\.rs:(186:18):" \
    --copy-vcs true \
    --cargo-test-arg --lib \
    --cargo-test-arg group_channel_crypto::tests::encrypt_requires_key_agreement_seed \
    --timeout "$timeout_seconds"

run_slice "core-http-transport" 3 \
  cargo mutants -p kamn-core \
    --file crates/kamn-core/src/kolme_runtime_commit/http_transport.rs \
    --re "http_transport\\.rs:(50:9|291:17):" \
    --copy-vcs true \
    --cargo-test-arg --lib \
    --cargo-test-arg spec_c0 \
    --timeout "$timeout_seconds"

run_slice "node-runtime-orchestration" 1 \
  cargo mutants -p kamn-node \
    --file crates/kamn-node/src/runtime_orchestration.rs \
    --re "runtime_orchestration\\.rs:(768:34):" \
    --copy-vcs true \
    --cargo-test-arg --bin \
    --cargo-test-arg kamn-node \
    --cargo-test-arg main_tests::runtime_tests::unit_full_supervisor_stop_contract_classifier_rejects_status_mismatch \
    --timeout "$timeout_seconds"

run_slice "node-service-api-endpoint" 2 \
  cargo mutants -p kamn-node \
    --file crates/kamn-node/src/service_api_endpoint.rs \
    --re "service_api_endpoint\\.rs:(336:9):" \
    --copy-vcs true \
    --cargo-test-arg --bin \
    --cargo-test-arg kamn-node \
    --cargo-test-arg main_tests::service_api_endpoint_tests::regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender \
    --timeout "$timeout_seconds"

run_slice "node-signer" 1 \
  cargo mutants -p kamn-node \
    --file crates/kamn-node/src/signer.rs \
    --re "signer\\.rs:(198:33):" \
    --copy-vcs true \
    --cargo-test-arg --bin \
    --cargo-test-arg kamn-node \
    --cargo-test-arg signer::tests::regression_strict_signer_secret_source_precedence_rejects_dual_private_key_envs \
    --timeout "$timeout_seconds"

python3 - "$slice_tsv" "$output_json" <<'PY'
import csv
import json
import sys
from pathlib import Path

schema_version = "kamn.ci.critical-path-mutation-report.v1"
reason_taxonomy_version = "kamn.ci.critical-path-mutation-reason-taxonomy.v1"
ordered_reason_codes = [
    "critical_path_mutation_slice_discovery_missing",
    "critical_path_mutation_slice_expected_count_mismatch",
    "critical_path_mutation_slice_summary_parse_failed",
    "critical_path_mutation_slice_exit_nonzero",
    "critical_path_mutation_slice_escape_detected",
]
order = {code: idx for idx, code in enumerate(ordered_reason_codes)}

slice_rows = []
totals = {
    "expected_mutants": 0,
    "discovered_mutants": 0,
    "tested_mutants": 0,
    "caught_mutants": 0,
    "missed_mutants": 0,
    "unviable_mutants": 0,
    "timeout_mutants": 0,
}
reason_codes: set[str] = set()

with Path(sys.argv[1]).open("r", encoding="utf-8", newline="") as handle:
    reader = csv.reader(handle, delimiter="\t")
    for row in reader:
        if len(row) != 11:
            continue
        (
            slice_id,
            expected,
            discovered,
            tested,
            caught,
            missed,
            unviable,
            timeout,
            exit_code,
            status,
            reasons_csv,
        ) = row
        entry = {
            "slice_id": slice_id,
            "expected_mutants": int(expected),
            "discovered_mutants": int(discovered),
            "tested_mutants": int(tested),
            "caught_mutants": int(caught),
            "missed_mutants": int(missed),
            "unviable_mutants": int(unviable),
            "timeout_mutants": int(timeout),
            "exit_code": int(exit_code),
            "status": status,
        }
        if reasons_csv != "none":
            reasons = [code for code in reasons_csv.split(",") if code]
            entry["reason_codes"] = reasons
            reason_codes.update(reasons)
        else:
            entry["reason_codes"] = []
        for key in totals:
            if key in entry:
                totals[key] += entry[key]
        slice_rows.append(entry)

ordered = sorted(reason_codes, key=lambda value: (order.get(value, len(order)), value))
status = "ok" if not ordered else "fail"
final_decision = "GO" if status == "ok" else "NO-GO"
report = {
    "schema_version": schema_version,
    "status": status,
    "final_decision": final_decision,
    "reason_taxonomy_version": reason_taxonomy_version,
    "reason_codes": ordered,
    "reason_codes_csv": ",".join(ordered) if ordered else "none",
    "slice_count": len(slice_rows),
    "totals": totals,
    "slices": slice_rows,
}
Path(sys.argv[2]).write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

print(f"status={status}")
print(f"final_decision={final_decision}")
print(f"reason_taxonomy_version={reason_taxonomy_version}")
print(f"reason_codes_csv={report['reason_codes_csv']}")
print(f"slice_count={len(slice_rows)}")
print(f"tested_mutants={totals['tested_mutants']}")
print(f"caught_mutants={totals['caught_mutants']}")
print(f"missed_mutants={totals['missed_mutants']}")
print(f"unviable_mutants={totals['unviable_mutants']}")
print(f"timeout_mutants={totals['timeout_mutants']}")
print(f"mutation_report_file={sys.argv[2]}")
sys.exit(0 if status == "ok" else 1)
PY
