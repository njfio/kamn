#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
SCRIPT="$ROOT_DIR/scripts/ci/check_legacy_ingress_parser_drift.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$SCRIPT" "expected legacy ingress parser drift checker to be executable"

SOURCE_ROOT="$TMP_DIR/src"
mkdir -p "$SOURCE_ROOT"
cat >"$SOURCE_ROOT/service_api_endpoint.rs" <<'EOF_RS'
use std::net::{TcpListener, TcpStream};

fn read_http_request() {}
fn parse_http_request_line() {}
pub(crate) fn serve_service_api_endpoint() {}
EOF_RS
cat >"$SOURCE_ROOT/main.rs" <<'EOF_RS'
fn main() {}
EOF_RS

BASELINE_FILE="$TMP_DIR/baseline.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$BASELINE_FILE" <<'EOF_BASELINE'
{
  "schema_version": "kamn.ci.legacy-ingress-parser-baseline.v1",
  "exclude_path_fragments": [],
  "markers": [
    {
      "id": "sync_http_request_reader",
      "pattern": "fn read_http_request(",
      "max_occurrences": 1,
      "allowed_files": ["service_api_endpoint.rs"]
    },
    {
      "id": "sync_http_request_line_parser",
      "pattern": "fn parse_http_request_line(",
      "max_occurrences": 1,
      "allowed_files": ["service_api_endpoint.rs"]
    },
    {
      "id": "sync_service_endpoint_server",
      "pattern": "pub(crate) fn serve_service_api_endpoint(",
      "max_occurrences": 1,
      "allowed_files": ["service_api_endpoint.rs"]
    }
  ]
}
EOF_BASELINE

pass_output="$(
  bash "$SCRIPT" \
    --source-root "$SOURCE_ROOT" \
    --baseline-file "$BASELINE_FILE"
)"

if ! printf '%s\n' "$pass_output" | grep -q '^status=pass$'; then
  echo "expected pass status for baseline-aligned parser drift report" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^policy_decision=GO$'; then
  echo "expected GO decision on baseline-aligned parser drift report" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes=none$'; then
  echo "expected reason_codes=none on baseline-aligned parser drift report" >&2
  exit 1
fi

cat >>"$SOURCE_ROOT/service_api_endpoint.rs" <<'EOF_RS'
fn parse_http_request_line() {}
EOF_RS

set +e
count_fail_output="$(
  bash "$SCRIPT" \
    --source-root "$SOURCE_ROOT" \
    --baseline-file "$BASELINE_FILE" 2>&1
)"
count_fail_code=$?
set -e

if [ "$count_fail_code" -eq 0 ]; then
  echo "expected checker to fail when legacy parser marker count increases" >&2
  exit 1
fi
if ! printf '%s\n' "$count_fail_output" | grep -q '^reason_codes=legacy_ingress_parser_marker_count_increased$'; then
  echo "expected count increase reason code when parser markers exceed baseline" >&2
  exit 1
fi

cat >"$SOURCE_ROOT/service_api_endpoint.rs" <<'EOF_RS'
use std::net::{TcpListener, TcpStream};

fn read_http_request() {}
fn parse_http_request_line() {}
pub(crate) fn serve_service_api_endpoint() {}
EOF_RS
cat >"$SOURCE_ROOT/other.rs" <<'EOF_RS'
fn parse_http_request_line() {}
EOF_RS

set +e
new_file_fail_output="$(
  bash "$SCRIPT" \
    --source-root "$SOURCE_ROOT" \
    --baseline-file "$BASELINE_FILE" 2>&1
)"
new_file_fail_code=$?
set -e

if [ "$new_file_fail_code" -eq 0 ]; then
  echo "expected checker to fail when legacy parser markers appear in non-allowed files" >&2
  exit 1
fi
if ! printf '%s\n' "$new_file_fail_output" | grep -q 'legacy_ingress_parser_marker_new_file'; then
  echo "expected new-file reason code when parser markers appear outside allowed file list" >&2
  exit 1
fi

set +e
missing_baseline_output="$(
  bash "$SCRIPT" \
    --source-root "$SOURCE_ROOT" \
    --baseline-file "$TMP_DIR/missing-baseline.json" 2>&1
)"
missing_baseline_code=$?
set -e

if [ "$missing_baseline_code" -eq 0 ]; then
  echo "expected checker to fail when baseline file is missing" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_baseline_output" | grep -q '^reason_codes=legacy_ingress_parser_baseline_missing$'; then
  echo "expected missing-baseline reason code" >&2
  exit 1
fi

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$BASELINE_FILE" <<'EOF_BASELINE'
{
  "schema_version": "bad-schema"
}
EOF_BASELINE

set +e
invalid_baseline_output="$(
  bash "$SCRIPT" \
    --source-root "$SOURCE_ROOT" \
    --baseline-file "$BASELINE_FILE" 2>&1
)"
invalid_baseline_code=$?
set -e

if [ "$invalid_baseline_code" -eq 0 ]; then
  echo "expected checker to fail when baseline schema is invalid" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_baseline_output" | grep -q '^reason_codes=legacy_ingress_parser_baseline_invalid$'; then
  echo "expected invalid-baseline reason code" >&2
  exit 1
fi

echo "legacy ingress parser drift checker tests passed."
