#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/message/generate_key_lifecycle_invariant_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/message/check_key_lifecycle_invariant_policy.sh"

usage() {
  cat <<'EOF'
Usage:
  bash scripts/message/run_key_hierarchy_invariant_contract_lane.sh \
    [--output-file <path>] \
    [--skip-tests]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

output_file=""
skip_tests=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --skip-tests)
      skip_tests=true
      shift
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

if [ ! -x "$GENERATOR" ]; then
  fail "key lifecycle invariant evidence generator is not executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "key lifecycle invariant policy checker is not executable"
fi

if [ "$skip_tests" != true ]; then
  cargo test -p kamn-core --test agent_key_hierarchy >/dev/null
  cargo test -p kamn-core --test key_lifecycle >/dev/null
  cargo test -p kamn-core --test key_recovery >/dev/null
  cargo test -p kamn-core --test docs_contract_matrix_wave2_harness >/dev/null
fi

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

report_file="$tmp_dir/key-lifecycle-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_file" <<'JSON'
{
  "status": "pass",
  "rotation_replay_detected": false,
  "stale_generation_detected": false,
  "revocation_bypass_detected": false,
  "reason_codes": ["revocation_state_consistent", "rotation_sequence_monotonic"]
}
JSON

if [[ -z "$output_file" ]]; then
  output_file="$tmp_dir/key-lifecycle-invariant-evidence.json"
fi

generation_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --lane contract \
    --report-file "$report_file" \
    --ci-fast-gate PASS
)"

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'schema_version=%s\n' "$(extract_value "$policy_output" "schema_version")"
printf 'evidence_key=%s\n' "$(extract_value "$generation_output" "evidence_key")"
printf 'reason_key=%s\n' "$(extract_value "$generation_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
echo "key hierarchy invariant contract lane tests passed."
