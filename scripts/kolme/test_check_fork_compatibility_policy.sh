#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
ROOT_DIR="$KAMN_ROOT"
GENERATOR="$ROOT_DIR/scripts/kolme/generate_fork_compatibility_evidence.py"
CHECKER="$ROOT_DIR/scripts/kolme/check_fork_compatibility_policy.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected fork compatibility evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected fork compatibility policy checker to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/fork-go-report.json"
python3 "$GENERATOR" \
  --upstream-release-tag "v0.15.2" \
  --fork-release-tag "v0.15.2" \
  --fork-repo "njfio/kolme_fork" \
  --fork-ref "refs/heads/main" \
  --ci-fast-gate PASS \
  --output-json "$go_report" \
  >/dev/null

go_policy_output="$(
  python3 "$CHECKER" \
    --report-file "$go_report" \
    --expected-upstream-release-tag "v0.15.2" \
    --expected-fork-release-tag "v0.15.2" \
    --expected-fork-repo "njfio/kolme_fork" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/policy-go-report.json"
)"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected policy checker to accept go report"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO policy decision for go report"

python3 - "$TMP_DIR/policy-go-report.json" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
if report.get("schema_version") != "kamn.kolme.fork-compatibility-policy-report.v1":
    raise SystemExit("unexpected fork compatibility policy report schema")
if report.get("reason_taxonomy_version") != "kamn.kolme.fork-compatibility-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason_taxonomy_version marker in policy report")
if report.get("reason_codes_csv") != "unsupported_upstream_major,unsupported_fork_major,upstream_minor_out_of_supported_window,fork_minor_out_of_supported_window,fork_release_tag_mismatch,fork_ref_missing,ci_fast_gate_failed":
    raise SystemExit("expected deterministic reason_codes_csv marker in policy report")
if report.get("upgrade_rehearsal_bypass_guard_status") != "verified":
    raise SystemExit("expected deterministic upgrade_rehearsal_bypass_guard_status marker in policy report")
PY

drift_report="$TMP_DIR/fork-drift-report.json"
set +e
python3 "$GENERATOR" \
  --upstream-release-tag "v0.15.2" \
  --fork-release-tag "v0.14.9" \
  --fork-repo "njfio/kolme_fork" \
  --fork-ref "refs/heads/main" \
  --ci-fast-gate PASS \
  --output-json "$drift_report" \
  >/dev/null
generator_code=$?
set -e
if [ "$generator_code" -eq 0 ]; then
  echo "expected drifted fork tuple generation to fail closed" >&2
  exit 1
fi

drift_policy_output="$(
  python3 "$CHECKER" \
    --report-file "$drift_report" \
    --expected-upstream-release-tag "v0.15.2" \
    --expected-fork-release-tag "v0.14.9" \
    --expected-fork-repo "njfio/kolme_fork" \
    --expected-final-decision NO-GO \
    --require-reason-code fork_release_tag_mismatch \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/policy-drift-report.json"
)"
assert_eq "$(extract_value "$drift_policy_output" "status")" "ok" "expected policy checker to accept expected no-go report"
assert_eq "$(extract_value "$drift_policy_output" "final_decision")" "GO" "expected GO policy decision when expected no-go reason code is present"

malformed_report="$TMP_DIR/malformed-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$malformed_report" <<'JSON'
{
  "schema_version": "kamn.kolme.fork-compatibility-report.v0",
  "upstream_release_tag": "v0.15.2",
  "fork_release_tag": "v0.15.2",
  "fork_repo": "njfio/kolme_fork",
  "fork_ref": "refs/heads/main",
  "reason_codes": [],
  "final_decision": "GO"
}
JSON

set +e
malformed_output="$(
  python3 "$CHECKER" \
    --report-file "$malformed_report" \
    --expected-upstream-release-tag "v0.15.2" \
    --expected-fork-release-tag "v0.15.2" \
    --expected-fork-repo "njfio/kolme_fork" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/policy-malformed-report.json" 2>&1
)"
malformed_code=$?
set -e
if [ "$malformed_code" -eq 0 ]; then
  echo "expected malformed report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$malformed_output" | grep -q "report_schema_invalid"; then
  echo "expected malformed report to emit report_schema_invalid reason code" >&2
  exit 1
fi

bypass_tampered_report="$TMP_DIR/fork-go-report.bypass-tampered.json"
python3 - "$go_report" "$bypass_tampered_report" <<'PY'
import json
import pathlib
import sys

source = json.loads(pathlib.Path(sys.argv[1]).read_text())
source["upgrade_rehearsal_bypass_guard_status"] = "tampered"
pathlib.Path(sys.argv[2]).write_text(json.dumps(source, sort_keys=True, indent=2) + "\n")
PY

set +e
bypass_tampered_output="$(
  python3 "$CHECKER" \
    --report-file "$bypass_tampered_report" \
    --expected-upstream-release-tag "v0.15.2" \
    --expected-fork-release-tag "v0.15.2" \
    --expected-fork-repo "njfio/kolme_fork" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/policy-bypass-tampered-report.json" 2>&1
)"
bypass_tampered_code=$?
set -e
if [ "$bypass_tampered_code" -eq 0 ]; then
  echo "expected upgrade-rehearsal bypass marker tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$bypass_tampered_output" | grep -q "report_upgrade_rehearsal_bypass_guard_status_mismatch"; then
  echo "expected deterministic upgrade-rehearsal bypass mismatch reason code" >&2
  exit 1
fi

python3 - "$TMP_DIR/policy-malformed-report.json" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
if report.get("schema_version") != "kamn.kolme.fork-compatibility-policy-report.v1":
    raise SystemExit("unexpected fork compatibility policy report schema")
if report.get("final_decision") != "NO-GO":
    raise SystemExit("expected malformed report policy decision to be NO-GO")
PY

# Regression: #1402
if ! printf '%s\n' "$malformed_output" | grep -q "report_schema_invalid"; then
  echo "expected malformed schema regression guard to remain fail-closed" >&2
  exit 1
fi

echo "Kolme fork compatibility policy checker tests passed."
