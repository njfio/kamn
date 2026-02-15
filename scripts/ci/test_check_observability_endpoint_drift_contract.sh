#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_observability_endpoint_drift_contract.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECK_SCRIPT" ]; then
  echo "expected observability endpoint drift checker wrapper to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/observability-endpoint-drift-report.json"
check_output="$(
  bash "$CHECK_SCRIPT" --output-json "$report_file"
)"
if ! printf '%s\n' "$check_output" | grep -q '^status=pass$'; then
  echo "expected observability endpoint drift checker status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$check_output" | grep -q '^final_decision=GO$'; then
  echo "expected observability endpoint drift checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$check_output" | grep -q '^observability_async_ingress_contract_status=verified$'; then
  echo "expected observability endpoint drift checker async-ingress status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$check_output" | grep -q '^observability_framework_parity_status=verified$'; then
  echo "expected observability endpoint drift checker framework parity status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$check_output" | grep -q '^docs_migration_contract_status=verified$'; then
  echo "expected observability endpoint drift checker docs status marker" >&2
  exit 1
fi

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.ci.observability-endpoint-drift-report.v1":
    raise SystemExit("unexpected observability endpoint drift report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("observability_async_ingress_contract_status") != "verified":
    raise SystemExit("expected observability_async_ingress_contract_status=verified")
if payload.get("observability_framework_parity_status") != "verified":
    raise SystemExit("expected observability_framework_parity_status=verified")
if payload.get("docs_migration_contract_status") != "verified":
    raise SystemExit("expected docs_migration_contract_status=verified")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected reason_codes=['none']")
PY

tampered_source="$TMP_DIR/observability_endpoint.rs"
cp "$ROOT_DIR/crates/kamn-node/src/observability_endpoint.rs" "$tampered_source"
python3 - "$tampered_source" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace(
    "async fn dispatch_observability_endpoint_request(",
    "fn dispatch_observability_endpoint_request(",
    1,
)
path.write_text(text, encoding="utf-8")
PY

set +e
tampered_source_output="$(
  bash "$CHECK_SCRIPT" --source-file "$tampered_source" --output-json "$TMP_DIR/observability-endpoint-drift-report.tampered-source.json" 2>&1
)"
tampered_source_code=$?
set -e
if [ "$tampered_source_code" -eq 0 ]; then
  echo "expected observability endpoint drift checker to fail on source marker drift" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_source_output" | grep -q 'observability_source_marker_missing:async_dispatch'; then
  echo "expected deterministic source-marker drift reason code" >&2
  exit 1
fi

tampered_docs="$TMP_DIR/node-runtime-cli.md"
cp "$ROOT_DIR/docs/foundation/node-runtime-cli.md" "$tampered_docs"
python3 - "$tampered_docs" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
needle = "Runtime observability endpoint ingress runs on async tokio listener path; drift contracts enforce fail-closed parity for unknown-path, malformed-request, and timeout compatibility behavior."
if needle not in text:
    raise SystemExit("expected docs marker not found in baseline copy")
path.write_text(text.replace(needle, "", 1), encoding="utf-8")
PY

set +e
tampered_docs_output="$(
  bash "$CHECK_SCRIPT" --docs-file "$tampered_docs" --output-json "$TMP_DIR/observability-endpoint-drift-report.tampered-docs.json" 2>&1
)"
tampered_docs_code=$?
set -e
if [ "$tampered_docs_code" -eq 0 ]; then
  echo "expected observability endpoint drift checker to fail on docs marker drift" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_docs_output" | grep -q 'observability_docs_marker_missing'; then
  echo "expected deterministic docs-marker drift reason code" >&2
  exit 1
fi

echo "observability endpoint drift checker tests passed."
