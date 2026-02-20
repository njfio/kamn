#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_shell_rust_ratio_guardrail.sh"

if [[ ! -x "$CHECKER" ]]; then
  echo "expected shell-rust ratio guardrail checker to be executable: $CHECKER" >&2
  exit 1
fi
if [[ "$(wc -l <"$CHECKER")" -gt 20 ]]; then
  echo "expected shell-rust ratio checker shell surface to stay <=20 lines: $CHECKER" >&2
  exit 1
fi
if ! grep -q "check_shell_rust_ratio_guardrail.py" "$CHECKER"; then
  echo "expected shell-rust ratio checker to delegate to python implementation: $CHECKER" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

PASS_THRESHOLD_FILE="$TMP_DIR/pass-thresholds.env"
cat > "$PASS_THRESHOLD_FILE" <<'EOF_PASS'
WARN_SHELL_RUST_RATIO_MAX=999
FAIL_SHELL_RUST_RATIO_MAX=1000
EOF_PASS

pass_output="$({
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --threshold-file "$PASS_THRESHOLD_FILE" \
    --output-json "$TMP_DIR/pass-report.json"
} 2>&1)"

if ! printf '%s\n' "$pass_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for permissive threshold path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^final_decision=GO$'; then
  echo "expected final_decision=GO for permissive threshold path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes=none$'; then
  echo "expected reason_codes=none for permissive threshold path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_taxonomy_version=kamn.ci.shell-rust-ratio-guardrail-reason-taxonomy.v1$'; then
  echo "expected deterministic reason taxonomy marker on permissive path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -Eq '^shell_to_rust_ratio=[0-9]+(\.[0-9]+)?$'; then
  echo "expected numeric shell_to_rust_ratio marker on permissive path" >&2
  exit 1
fi

WARN_THRESHOLD_FILE="$TMP_DIR/warn-thresholds.env"
cat > "$WARN_THRESHOLD_FILE" <<'EOF_WARN'
WARN_SHELL_RUST_RATIO_MAX=0.10
FAIL_SHELL_RUST_RATIO_MAX=1000
EOF_WARN

warn_output="$({
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --threshold-file "$WARN_THRESHOLD_FILE" \
    --output-json "$TMP_DIR/warn-report.json"
} 2>&1)"

if ! printf '%s\n' "$warn_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for warn-only threshold path" >&2
  exit 1
fi
if ! printf '%s\n' "$warn_output" | grep -q '^final_decision=WARN$'; then
  echo "expected final_decision=WARN for warn-only threshold path" >&2
  exit 1
fi
if ! printf '%s\n' "$warn_output" | grep -q '^reason_codes=shell_rust_ratio_warn_threshold_exceeded$'; then
  echo "expected deterministic warn threshold reason marker" >&2
  exit 1
fi

FAIL_THRESHOLD_FILE="$TMP_DIR/fail-thresholds.env"
cat > "$FAIL_THRESHOLD_FILE" <<'EOF_FAIL'
WARN_SHELL_RUST_RATIO_MAX=0.01
FAIL_SHELL_RUST_RATIO_MAX=0.10
EOF_FAIL

set +e
fail_output="$({
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --threshold-file "$FAIL_THRESHOLD_FILE" \
    --output-json "$TMP_DIR/fail-report.json"
} 2>&1)"
fail_exit=$?
set -e

if [[ "$fail_exit" -eq 0 ]]; then
  echo "expected checker to fail when fail threshold is exceeded" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^status=fail$'; then
  echo "expected status=fail when fail threshold is exceeded" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected final_decision=NO-GO when fail threshold is exceeded" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^reason_codes=shell_rust_ratio_fail_threshold_exceeded$'; then
  echo "expected deterministic fail threshold reason marker" >&2
  exit 1
fi

MISSING_KEY_THRESHOLD_FILE="$TMP_DIR/missing-key-thresholds.env"
cat > "$MISSING_KEY_THRESHOLD_FILE" <<'EOF_MISSING'
WARN_SHELL_RUST_RATIO_MAX=0.90
EOF_MISSING

set +e
missing_key_output="$({
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --threshold-file "$MISSING_KEY_THRESHOLD_FILE" \
    --output-json "$TMP_DIR/missing-key-report.json"
} 2>&1)"
missing_key_exit=$?
set -e

if [[ "$missing_key_exit" -eq 0 ]]; then
  echo "expected checker to fail when required threshold key is missing" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_key_output" | grep -q '^reason_codes=shell_rust_ratio_threshold_key_missing$'; then
  echo "expected deterministic missing-key reason marker" >&2
  exit 1
fi

ORDER_THRESHOLD_FILE="$TMP_DIR/order-thresholds.env"
cat > "$ORDER_THRESHOLD_FILE" <<'EOF_ORDER'
WARN_SHELL_RUST_RATIO_MAX=1.00
FAIL_SHELL_RUST_RATIO_MAX=1.00
EOF_ORDER

set +e
order_output="$({
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --threshold-file "$ORDER_THRESHOLD_FILE" \
    --output-json "$TMP_DIR/order-report.json"
} 2>&1)"
order_exit=$?
set -e

if [[ "$order_exit" -eq 0 ]]; then
  echo "expected checker to fail when threshold order is invalid" >&2
  exit 1
fi
if ! printf '%s\n' "$order_output" | grep -q '^reason_codes=shell_rust_ratio_threshold_order_invalid$'; then
  echo "expected deterministic threshold-order reason marker" >&2
  exit 1
fi

echo "shell-rust ratio guardrail checker tests passed."
