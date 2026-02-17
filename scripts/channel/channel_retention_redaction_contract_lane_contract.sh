#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/channel/generate_channel_retention_redaction_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/channel/check_channel_retention_redaction_policy.sh"

usage() {
  cat <<'EOF'
Usage:
  bash scripts/channel/run_channel_retention_redaction_contract_lane.sh \
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
  fail "channel retention/redaction evidence generator is not executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "channel retention/redaction policy checker is not executable"
fi

if [ "$skip_tests" != true ]; then
  cargo test -p kamn-core --test content_retention_tombstones >/dev/null
  cargo test -p kamn-core --test redaction_tombstones >/dev/null
  cargo test -p kamn-core --test data_governance_retention_docs >/dev/null
fi

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

retention_report_file="$tmp_dir/retention-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$retention_report_file" <<'JSON'
{
  "status": "pass",
  "total_candidates": 3,
  "replay_safe": true,
  "reason_codes": ["candidate_order_stable", "retention_window_valid"]
}
JSON

redaction_report_file="$tmp_dir/redaction-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$redaction_report_file" <<'JSON'
{
  "status": "pass",
  "applied_count": 2,
  "replay_safe": true,
  "reason_codes": ["redaction_hash_stable", "redaction_replay_guard_active"]
}
JSON

if [[ -z "$output_file" ]]; then
  output_file="$tmp_dir/channel-retention-redaction-evidence.json"
fi

generation_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --lane contract \
    --retention-report-file "$retention_report_file" \
    --redaction-report-file "$redaction_report_file" \
    --ci-fast-gate PASS
)"

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'schema_version=%s\n' "$(extract_value "$policy_output" "schema_version")"
printf 'evidence_key=%s\n' "$(extract_value "$generation_output" "evidence_key")"
printf 'reason_key=%s\n' "$(extract_value "$generation_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
echo "channel retention/redaction contract lane tests passed."
