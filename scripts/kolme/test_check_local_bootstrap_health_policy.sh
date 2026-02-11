#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_bootstrap_health_policy.py"
TMP_DIR="$(mktemp -d)"
TMP_REPORT_OK="$TMP_DIR/ok-report.json"
TMP_REPORT_BAD="$TMP_DIR/bad-report.json"
TMP_POLICY_OUT="$TMP_DIR/policy-report.json"
TMP_ERR="$TMP_DIR/error.log"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected local bootstrap health policy checker to be executable" >&2
  exit 1
fi

cat >"$TMP_REPORT_OK" <<'JSON'
{
  "schema_version": "kamn.kolme.local-bootstrap-summary.v1",
  "mode": "dry-run",
  "status": "ok",
  "ready": false,
  "readiness_status": "planned",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "checks": [
    {
      "id": "version_compatibility",
      "command": "python3 scripts/kolme/validate_version_compatibility.py --kamn-version 1.1.0 --kolme-release-tag v0.15.2 --ci-fast-gate PASS --output-json /tmp/kolme-bootstrap-version-report.json",
      "status": "planned"
    },
    {
      "id": "fork_compatibility_evidence",
      "command": "python3 scripts/kolme/generate_fork_compatibility_evidence.py --upstream-release-tag v0.15.2 --fork-release-tag v0.15.2 --fork-repo njfio/kolme_fork --fork-ref refs/heads/main --ci-fast-gate PASS --output-json /tmp/kolme-bootstrap-fork-compatibility-report.json",
      "status": "planned"
    },
    {
      "id": "fork_compatibility_policy",
      "command": "python3 scripts/kolme/check_fork_compatibility_policy.py --report-file /tmp/kolme-bootstrap-fork-compatibility-report.json --expected-upstream-release-tag v0.15.2 --expected-fork-release-tag v0.15.2 --expected-fork-repo njfio/kolme_fork --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-bootstrap-fork-compatibility-policy-report.json",
      "status": "planned"
    },
    {
      "id": "triadic_devnet_smoke",
      "command": "bash scripts/kolme/run_triadic_devnet_smoke.sh --output-file /tmp/kolme-bootstrap-devnet-markers.txt",
      "status": "planned"
    },
    {
      "id": "triadic_devnet_validate",
      "command": "python3 scripts/kolme/validate_triadic_devnet_smoke.py --fixture fixtures/kolme_compatibility/devnet_smoke_markers.json --marker-file /tmp/kolme-bootstrap-devnet-markers.txt --output-json /tmp/kolme-bootstrap-devnet-report.json",
      "status": "planned"
    }
  ],
  "artifact_paths": [
    "/tmp/kolme-bootstrap-version-report.json",
    "/tmp/kolme-bootstrap-fork-compatibility-report.json",
    "/tmp/kolme-bootstrap-fork-compatibility-policy-report.json",
    "/tmp/kolme-bootstrap-devnet-markers.txt",
    "/tmp/kolme-bootstrap-devnet-report.json"
  ]
}
JSON

python3 "$CHECKER" \
  --report-file "$TMP_REPORT_OK" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$TMP_POLICY_OUT" >/dev/null

python3 - "$TMP_POLICY_OUT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-bootstrap-policy-report.v1":
    raise SystemExit("unexpected local bootstrap health policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid local bootstrap report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no failed checks for valid local bootstrap report")
PY

cat >"$TMP_REPORT_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.local-bootstrap-summary.v1",
  "mode": "dry-run",
  "status": "ok",
  "ready": false,
  "readiness_status": "planned",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "checks": [
    {
      "id": "version_compatibility",
      "command": "python3 scripts/kolme/validate_version_compatibility.py --kamn-version 1.1.0 --kolme-release-tag v0.15.2 --ci-fast-gate PASS --output-json /tmp/kolme-bootstrap-version-report.json",
      "status": "planned"
    }
  ],
  "artifact_paths": [
    "/tmp/kolme-bootstrap-version-report.json"
  ]
}
JSON

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_BAD" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$TMP_POLICY_OUT" >"$TMP_ERR" 2>&1
bad_exit_code=$?
set -e

if [ "$bad_exit_code" -eq 0 ]; then
  echo "expected policy checker to fail for missing local bootstrap checks" >&2
  exit 1
fi

if ! grep -q "check_missing:triadic_devnet_smoke" "$TMP_ERR"; then
  echo "expected missing triadic devnet smoke marker for policy failure" >&2
  exit 1
fi

echo "local bootstrap health policy checker tests passed."
