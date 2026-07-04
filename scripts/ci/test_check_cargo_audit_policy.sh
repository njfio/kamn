#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CHECKER="$ROOT_DIR/scripts/ci/check_cargo_audit_policy.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$CHECKER" "expected cargo-audit policy checker to be executable"

PASS_REPORT="$TMP_DIR/cargo-audit-pass.json"
HIGH_REPORT="$TMP_DIR/cargo-audit-high.json"
UNKNOWN_REPORT="$TMP_DIR/cargo-audit-unknown.json"
WARNING_REPORT="$TMP_DIR/cargo-audit-warning.json"
WAIVER_EMPTY="$TMP_DIR/cargo-audit-waivers-empty.json"
WAIVER_VALID="$TMP_DIR/cargo-audit-waivers-valid.json"
WAIVER_WARNING_VALID="$TMP_DIR/cargo-audit-waivers-warning-valid.json"
WAIVER_UNKNOWN_VALID="$TMP_DIR/cargo-audit-waivers-unknown-valid.json"
WAIVER_INVALID_TRACKING="$TMP_DIR/cargo-audit-waivers-invalid-tracking.json"
PASS_OUTPUT_JSON="$TMP_DIR/cargo-audit-policy-pass.json"
WAIVED_OUTPUT_JSON="$TMP_DIR/cargo-audit-policy-waived.json"
WAIVED_WARNING_OUTPUT_JSON="$TMP_DIR/cargo-audit-policy-waived-warning.json"
WAIVED_UNKNOWN_OUTPUT_JSON="$TMP_DIR/cargo-audit-policy-waived-unknown.json"

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$PASS_REPORT" <<'JSON'
{
  "advisories": {
    "list": [
      {
        "advisory": {
          "id": "RUSTSEC-2026-1001",
          "severity": "low"
        },
        "package": {
          "name": "safe-low",
          "version": "1.0.0"
        }
      },
      {
        "advisory": {
          "id": "RUSTSEC-2026-1002",
          "severity": "moderate"
        },
        "package": {
          "name": "safe-moderate",
          "version": "1.0.0"
        }
      }
    ]
  }
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$HIGH_REPORT" <<'JSON'
{
  "advisories": {
    "list": [
      {
        "advisory": {
          "id": "RUSTSEC-2026-2001",
          "severity": "high"
        },
        "package": {
          "name": "risky-high",
          "version": "2.0.0"
        }
      }
    ],
    "yanked": [
      {
        "package": {
          "name": "yanked-warning",
          "version": "4.1.0"
        }
      }
    ]
  }
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$UNKNOWN_REPORT" <<'JSON'
{
  "advisories": {
    "list": [
      {
        "advisory": {
          "id": "RUSTSEC-2026-3001",
          "severity": "unscored"
        },
        "package": {
          "name": "unknown-severity",
          "version": "3.0.0"
        }
      }
    ]
  }
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$WARNING_REPORT" <<'JSON'
{
  "vulnerabilities": {
    "found": false,
    "count": 0,
    "list": []
  },
  "warnings": {
    "unmaintained": [
      {
        "advisory": {
          "id": "RUSTSEC-2026-4001",
          "informational": "unmaintained"
        },
        "package": {
          "name": "stale-warning",
          "version": "4.0.0"
        }
      }
    ],
    "yanked": [
      {
        "package": {
          "name": "yanked-warning",
          "version": "4.1.0"
        }
      }
    ]
  }
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$WAIVER_EMPTY" <<'JSON'
{
  "schema_version": "kamn.ci.cargo-audit-waiver.v1",
  "waivers": []
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$WAIVER_VALID" <<'JSON'
{
  "schema_version": "kamn.ci.cargo-audit-waiver.v1",
  "waivers": [
    {
      "advisory_id": "RUSTSEC-2026-2001",
      "package": "risky-high",
      "reason": "Temporary exception while upstream patch is pending.",
      "tracking_issue": "#5941",
      "expires_on": "2099-12-31"
    }
  ]
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$WAIVER_WARNING_VALID" <<'JSON'
{
  "schema_version": "kamn.ci.cargo-audit-waiver.v1",
  "waivers": [
    {
      "advisory_id": "RUSTSEC-2026-4001",
      "package": "stale-warning",
      "reason": "Temporary exception while upstream replacement is tracked.",
      "tracking_issue": "#7032",
      "expires_on": "2099-12-31"
    },
    {
      "advisory_id": "cargo-audit-yanked:yanked-warning",
      "package": "yanked-warning",
      "reason": "Temporary exception while upstream replacement is tracked.",
      "tracking_issue": "#7032",
      "expires_on": "2099-12-31"
    }
  ]
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$WAIVER_UNKNOWN_VALID" <<'JSON'
{
  "schema_version": "kamn.ci.cargo-audit-waiver.v1",
  "waivers": [
    {
      "advisory_id": "RUSTSEC-2026-3001",
      "package": "unknown-severity",
      "reason": "Temporary exception while upstream severity metadata is tracked.",
      "tracking_issue": "#7032",
      "expires_on": "2099-12-31"
    }
  ]
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$WAIVER_INVALID_TRACKING" <<'JSON'
{
  "schema_version": "kamn.ci.cargo-audit-waiver.v1",
  "waivers": [
    {
      "advisory_id": "RUSTSEC-2026-2001",
      "package": "risky-high",
      "reason": "Invalid tracking format for regression test.",
      "tracking_issue": "5941",
      "expires_on": "2099-12-31"
    }
  ]
}
JSON

pass_output="$(
  python3 "$CHECKER" \
    --audit-json "$PASS_REPORT" \
    --waiver-file "$WAIVER_EMPTY" \
    --threshold-max-severity moderate \
    --as-of-date 2026-02-25 \
    --output-json "$PASS_OUTPUT_JSON"
)"
if ! printf '%s\n' "$pass_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for pass report" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes_csv=none$'; then
  echo "expected reason_codes_csv=none for pass report" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^review_required=false$'; then
  echo "expected review_required=false for pass report" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^threshold_exceeded_total=0$'; then
  echo "expected threshold_exceeded_total=0 for pass report" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^policy_elapsed_seconds='; then
  echo "expected policy_elapsed_seconds marker for pass report" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi

if python3 "$CHECKER" \
  --audit-json "$HIGH_REPORT" \
  --waiver-file "$WAIVER_EMPTY" \
  --threshold-max-severity moderate \
  --as-of-date 2026-02-25 \
  >"$TMP_DIR/high-unwaived.out" \
  2>"$TMP_DIR/high-unwaived.err"
then
  echo "expected checker to fail on unwaived high severity advisory" >&2
  cat "$TMP_DIR/high-unwaived.out" >&2 || true
  cat "$TMP_DIR/high-unwaived.err" >&2 || true
  exit 1
fi
if ! grep -q '^reason_codes_csv=cargo_audit_advisory_threshold_exceeded_unwaived$' "$TMP_DIR/high-unwaived.out"; then
  echo "expected deterministic unwaived threshold reason code" >&2
  cat "$TMP_DIR/high-unwaived.out" >&2 || true
  exit 1
fi
if ! grep -q '^review_required=false$' "$TMP_DIR/high-unwaived.out"; then
  echo "expected review_required=false for unwaived threshold failure" >&2
  cat "$TMP_DIR/high-unwaived.out" >&2 || true
  exit 1
fi

waived_output="$(
  python3 "$CHECKER" \
    --audit-json "$HIGH_REPORT" \
    --waiver-file "$WAIVER_VALID" \
    --threshold-max-severity moderate \
    --as-of-date 2026-02-25 \
    --output-json "$WAIVED_OUTPUT_JSON"
)"
if ! printf '%s\n' "$waived_output" | grep -q '^status=ok$'; then
  echo "expected status=ok with valid waiver" >&2
  printf '%s\n' "$waived_output" >&2
  exit 1
fi
if ! printf '%s\n' "$waived_output" | grep -q '^reason_codes_csv=cargo_audit_advisory_threshold_exceeded_waived$'; then
  echo "expected deterministic waived threshold reason code" >&2
  printf '%s\n' "$waived_output" >&2
  exit 1
fi
if ! printf '%s\n' "$waived_output" | grep -q '^review_required=true$'; then
  echo "expected review_required=true for waived threshold advisory" >&2
  printf '%s\n' "$waived_output" >&2
  exit 1
fi
python3 - "$WAIVED_OUTPUT_JSON" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.ci.cargo-audit-policy-report.v1":
    raise SystemExit("expected deterministic schema_version marker")
if payload.get("waived_total") != 1:
    raise SystemExit("expected waived_total=1")
if payload.get("unwaived_total") != 0:
    raise SystemExit("expected unwaived_total=0")
if payload.get("review_required") is not True:
    raise SystemExit("expected review_required=true in report JSON")
if not isinstance(payload.get("policy_elapsed_seconds"), (float, int)):
    raise SystemExit("expected numeric policy_elapsed_seconds marker in report JSON")
PY

if python3 "$CHECKER" \
  --audit-json "$HIGH_REPORT" \
  --waiver-file "$WAIVER_INVALID_TRACKING" \
  --threshold-max-severity moderate \
  --as-of-date 2026-02-25 \
  >"$TMP_DIR/invalid-tracking.out" \
  2>"$TMP_DIR/invalid-tracking.err"
then
  echo "expected checker to fail when waiver tracking_issue is invalid" >&2
  cat "$TMP_DIR/invalid-tracking.out" >&2 || true
  cat "$TMP_DIR/invalid-tracking.err" >&2 || true
  exit 1
fi
if ! grep -q 'cargo_audit_waiver_tracking_issue_invalid' "$TMP_DIR/invalid-tracking.out"; then
  echo "expected waiver tracking issue reason code on invalid waiver input" >&2
  cat "$TMP_DIR/invalid-tracking.out" >&2 || true
  exit 1
fi

if python3 "$CHECKER" \
  --audit-json "$UNKNOWN_REPORT" \
  --waiver-file "$WAIVER_EMPTY" \
  --threshold-max-severity moderate \
  --as-of-date 2026-02-25 \
  >"$TMP_DIR/unknown-severity.out" \
  2>"$TMP_DIR/unknown-severity.err"
then
  echo "expected checker to fail on unknown advisory severity" >&2
  cat "$TMP_DIR/unknown-severity.out" >&2 || true
  cat "$TMP_DIR/unknown-severity.err" >&2 || true
  exit 1
fi
if ! grep -q 'cargo_audit_advisory_severity_unknown' "$TMP_DIR/unknown-severity.out"; then
  echo "expected unknown severity reason code" >&2
  cat "$TMP_DIR/unknown-severity.out" >&2 || true
  exit 1
fi

waived_unknown_output="$(
  python3 "$CHECKER" \
    --audit-json "$UNKNOWN_REPORT" \
    --waiver-file "$WAIVER_UNKNOWN_VALID" \
    --threshold-max-severity moderate \
    --as-of-date 2026-02-25 \
    --output-json "$WAIVED_UNKNOWN_OUTPUT_JSON"
)"
if ! printf '%s\n' "$waived_unknown_output" | grep -q '^status=ok$'; then
  echo "expected status=ok with valid unknown-severity waiver" >&2
  printf '%s\n' "$waived_unknown_output" >&2
  exit 1
fi
if ! printf '%s\n' "$waived_unknown_output" | grep -q 'cargo_audit_advisory_unknown_severity_waived'; then
  echo "expected deterministic waived unknown-severity reason code" >&2
  printf '%s\n' "$waived_unknown_output" >&2
  exit 1
fi
if ! printf '%s\n' "$waived_unknown_output" | grep -q '^unknown_severity_waived_total=1$'; then
  echo "expected unknown_severity_waived_total=1" >&2
  printf '%s\n' "$waived_unknown_output" >&2
  exit 1
fi
python3 - "$WAIVED_UNKNOWN_OUTPUT_JSON" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("unknown_severity_total") != 0:
    raise SystemExit("expected unknown_severity_total=0")
if payload.get("unknown_severity_waived_total") != 1:
    raise SystemExit("expected unknown_severity_waived_total=1")
if payload.get("review_required") is not True:
    raise SystemExit("expected review_required=true for unknown-severity waiver")
PY

if python3 "$CHECKER" \
  --audit-json "$WARNING_REPORT" \
  --waiver-file "$WAIVER_EMPTY" \
  --threshold-max-severity moderate \
  --as-of-date 2026-02-25 \
  >"$TMP_DIR/warning-unwaived.out" \
  2>"$TMP_DIR/warning-unwaived.err"
then
  echo "expected checker to fail on unwaived cargo-audit warning advisory" >&2
  cat "$TMP_DIR/warning-unwaived.out" >&2 || true
  cat "$TMP_DIR/warning-unwaived.err" >&2 || true
  exit 1
fi
if ! grep -q 'cargo_audit_warning_unwaived' "$TMP_DIR/warning-unwaived.out"; then
  echo "expected deterministic unwaived warning reason code" >&2
  cat "$TMP_DIR/warning-unwaived.out" >&2 || true
  exit 1
fi

waived_warning_output="$(
  python3 "$CHECKER" \
    --audit-json "$WARNING_REPORT" \
    --waiver-file "$WAIVER_WARNING_VALID" \
    --threshold-max-severity moderate \
    --as-of-date 2026-02-25 \
    --output-json "$WAIVED_WARNING_OUTPUT_JSON"
)"
if ! printf '%s\n' "$waived_warning_output" | grep -q '^status=ok$'; then
  echo "expected status=ok with valid warning waiver" >&2
  printf '%s\n' "$waived_warning_output" >&2
  exit 1
fi
if ! printf '%s\n' "$waived_warning_output" | grep -q '^reason_codes_csv=cargo_audit_warning_waived$'; then
  echo "expected deterministic waived warning reason code" >&2
  printf '%s\n' "$waived_warning_output" >&2
  exit 1
fi
if ! printf '%s\n' "$waived_warning_output" | grep -q '^warning_total=2$'; then
  echo "expected warning_total=2 for warning report" >&2
  printf '%s\n' "$waived_warning_output" >&2
  exit 1
fi
python3 - "$WAIVED_WARNING_OUTPUT_JSON" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("warning_total") != 2:
    raise SystemExit("expected warning_total=2")
if payload.get("waived_warning_total") != 2:
    raise SystemExit("expected waived_warning_total=2")
if payload.get("unwaived_warning_total") != 0:
    raise SystemExit("expected unwaived_warning_total=0")
if payload.get("review_required") is not True:
    raise SystemExit("expected review_required=true for warning waiver")
PY

echo "cargo-audit policy checker tests passed."
