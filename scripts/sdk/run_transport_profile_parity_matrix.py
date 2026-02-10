#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Dict, List


ROOT_DIR = Path(__file__).resolve().parents[2]
LANGUAGE_ORDER = ["rust", "python", "typescript"]
RUNNERS = {
    "rust": ["bash", str(ROOT_DIR / "scripts/sdk/run_transport_profile_probe_rust.sh")],
    "python": ["bash", str(ROOT_DIR / "scripts/sdk/run_transport_profile_probe_python.sh")],
    "typescript": [
        "bash",
        str(ROOT_DIR / "scripts/sdk/run_transport_profile_probe_typescript.sh"),
    ],
}
BACKEND_ADAPTER_PARITY_LANGUAGES = {"python", "typescript"}
BACKEND_ADAPTER_EXPECTED_KEYS = (
    "backend_adapter_register_id",
    "backend_adapter_message_id",
    "backend_adapter_receive_body",
    "backend_adapter_invalid_response_message",
    "backend_adapter_error_operation",
    "backend_adapter_error_reason",
    "backend_adapter_policy_reason",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--languages", default="all")
    parser.add_argument("--output-json", default="")
    parser.add_argument(
        "--backend-adapter-fixture",
        default=str(
            ROOT_DIR / "fixtures/sdk_parity/live_backend_adapter_profile_expectations.json"
        ),
    )
    parser.add_argument("--expect-default-mode", default="in-memory")
    parser.add_argument("--expect-live-mode", default="live")
    parser.add_argument("--expect-memory-mismatch-expected", default="live")
    parser.add_argument("--expect-memory-mismatch-found", default="in-memory")
    parser.add_argument("--expect-live-mismatch-expected", default="in-memory")
    parser.add_argument("--expect-live-mismatch-found", default="live")
    parser.add_argument("--expect-adapter-register-id", default="")
    parser.add_argument("--expect-adapter-message-id", default="")
    parser.add_argument("--expect-adapter-receive-body", default="")
    parser.add_argument("--expect-adapter-invalid-response-message", default="")
    parser.add_argument("--expect-adapter-error-operation", default="")
    parser.add_argument("--expect-adapter-error-reason", default="")
    parser.add_argument("--expect-adapter-policy-reason", default="")
    return parser.parse_args()


def parse_languages(raw: str) -> List[str]:
    normalized = raw.strip().lower()
    if normalized in {"", "all"}:
        return list(LANGUAGE_ORDER)

    selected: List[str] = []
    seen = set()
    for token in normalized.split(","):
        language = token.strip()
        if not language:
            continue
        if language not in RUNNERS:
            raise ValueError(f"unsupported language selector: {language}")
        if language not in seen:
            seen.add(language)
            selected.append(language)

    if not selected:
        raise ValueError("at least one language must be selected")
    return selected


def parse_key_values(stdout: str) -> Dict[str, str]:
    result: Dict[str, str] = {}
    for raw_line in stdout.splitlines():
        line = raw_line.strip()
        if not line or "=" not in line:
            continue
        key, value = line.split("=", 1)
        result[key] = value
    return result


def sanitize(value: str) -> str:
    return value.replace("\n", " ").strip()


def load_backend_adapter_fixture(path: str) -> Dict[str, str]:
    fixture_path = Path(path)
    try:
        raw = fixture_path.read_text(encoding="utf-8")
    except OSError as error:
        raise ValueError(f"unable to read backend adapter fixture: {error}") from error

    try:
        parsed = json.loads(raw)
    except json.JSONDecodeError as error:
        raise ValueError(
            f"invalid backend adapter fixture JSON: {error.msg}"
        ) from error

    if not isinstance(parsed, dict):
        raise ValueError("backend adapter fixture must be a JSON object")

    normalized: Dict[str, str] = {}
    for key in BACKEND_ADAPTER_EXPECTED_KEYS:
        value = parsed.get(key)
        if not isinstance(value, str) or value == "":
            raise ValueError(
                f"backend adapter fixture missing non-empty string for key: {key}"
            )
        normalized[key] = value
    return normalized


def run_runner(language: str) -> Dict[str, str]:
    completed = subprocess.run(
        RUNNERS[language],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
    )

    if completed.returncode != 0:
        message = sanitize(completed.stderr or completed.stdout or "runner failed")
        return {
            "status": "error",
            "error": message,
        }

    values = parse_key_values(completed.stdout)
    status = values.get("status", "error")
    if status != "ok":
        return {
            "status": "error",
            "error": values.get("error", "runner returned non-ok status"),
        }

    return {
        "status": "ok",
        "default_transport_mode": values.get("default_transport_mode", ""),
        "live_transport_mode": values.get("live_transport_mode", ""),
        "memory_mismatch_expected": values.get("memory_mismatch_expected", ""),
        "memory_mismatch_found": values.get("memory_mismatch_found", ""),
        "live_mismatch_expected": values.get("live_mismatch_expected", ""),
        "live_mismatch_found": values.get("live_mismatch_found", ""),
        "backend_adapter_register_id": values.get("backend_adapter_register_id", ""),
        "backend_adapter_message_id": values.get("backend_adapter_message_id", ""),
        "backend_adapter_receive_body": values.get("backend_adapter_receive_body", ""),
        "backend_adapter_invalid_response_message": values.get(
            "backend_adapter_invalid_response_message", ""
        ),
        "backend_adapter_error_operation": values.get(
            "backend_adapter_error_operation", ""
        ),
        "backend_adapter_error_reason": values.get("backend_adapter_error_reason", ""),
        "backend_adapter_policy_reason": values.get("backend_adapter_policy_reason", ""),
    }


def expected_for_language(
    language: str, base_expected: Dict[str, str], adapter_expected: Dict[str, str]
) -> Dict[str, str]:
    expected = dict(base_expected)
    if language in BACKEND_ADAPTER_PARITY_LANGUAGES:
        expected.update(adapter_expected)
    return expected


def main() -> int:
    args = parse_args()
    try:
        languages = parse_languages(args.languages)
    except ValueError as error:
        print(f"status=fail; reason={sanitize(str(error))}")
        return 2

    try:
        backend_adapter_fixture = load_backend_adapter_fixture(
            args.backend_adapter_fixture
        )
    except ValueError as error:
        print(f"status=fail; reason={sanitize(str(error))}")
        return 2

    base_expected = {
        "default_transport_mode": args.expect_default_mode,
        "live_transport_mode": args.expect_live_mode,
        "memory_mismatch_expected": args.expect_memory_mismatch_expected,
        "memory_mismatch_found": args.expect_memory_mismatch_found,
        "live_mismatch_expected": args.expect_live_mismatch_expected,
        "live_mismatch_found": args.expect_live_mismatch_found,
    }
    adapter_expected = {
        "backend_adapter_register_id": (
            args.expect_adapter_register_id
            or backend_adapter_fixture["backend_adapter_register_id"]
        ),
        "backend_adapter_message_id": (
            args.expect_adapter_message_id
            or backend_adapter_fixture["backend_adapter_message_id"]
        ),
        "backend_adapter_receive_body": (
            args.expect_adapter_receive_body
            or backend_adapter_fixture["backend_adapter_receive_body"]
        ),
        "backend_adapter_invalid_response_message": (
            args.expect_adapter_invalid_response_message
            or backend_adapter_fixture["backend_adapter_invalid_response_message"]
        ),
        "backend_adapter_error_operation": (
            args.expect_adapter_error_operation
            or backend_adapter_fixture["backend_adapter_error_operation"]
        ),
        "backend_adapter_error_reason": (
            args.expect_adapter_error_reason
            or backend_adapter_fixture["backend_adapter_error_reason"]
        ),
        "backend_adapter_policy_reason": (
            args.expect_adapter_policy_reason
            or backend_adapter_fixture["backend_adapter_policy_reason"]
        ),
    }

    failed_reasons: List[str] = []
    results: Dict[str, Dict[str, str]] = {}

    for language in languages:
        result = run_runner(language)
        results[language] = result
        if result.get("status") != "ok":
            failed_reasons.append(
                f"{language}: runner error ({result.get('error', 'unknown')})"
            )
            continue

        expected = expected_for_language(language, base_expected, adapter_expected)
        for key, expected_value in expected.items():
            actual_value = result.get(key, "")
            if actual_value != expected_value:
                failed_reasons.append(
                    f"{language}: {key} expected {expected_value} got {actual_value}"
                )

    if len(languages) > 1:
        for index, left_language in enumerate(languages):
            left = results.get(left_language, {})
            if left.get("status") != "ok":
                continue
            left_expected = expected_for_language(
                left_language, base_expected, adapter_expected
            )
            for right_language in languages[index + 1 :]:
                right = results.get(right_language, {})
                if right.get("status") != "ok":
                    continue
                right_expected = expected_for_language(
                    right_language, base_expected, adapter_expected
                )
                shared_keys = set(left_expected.keys()).intersection(
                    right_expected.keys()
                )
                for key in sorted(shared_keys):
                    if left.get(key, "") != right.get(key, ""):
                        failed_reasons.append(
                            f"{right_language}: parity mismatch for {key} against "
                            f"{left_language}"
                        )

    status = "pass" if not failed_reasons else "fail"
    report = {
        "status": status,
        "languages": languages,
        "expected": {
            "base": base_expected,
            "backend_adapter": adapter_expected,
            "backend_adapter_languages": sorted(BACKEND_ADAPTER_PARITY_LANGUAGES),
            "backend_adapter_fixture": args.backend_adapter_fixture,
        },
        "failed_reasons": failed_reasons,
        "results": results,
    }

    if args.output_json:
        Path(args.output_json).write_text(json.dumps(report, indent=2), encoding="utf-8")

    if status == "pass":
        print(
            "status=pass; "
            f"languages={','.join(languages)}; "
            f"checks_base={len(base_expected)}; "
            f"checks_backend_adapter={len(adapter_expected)}"
        )
        return 0

    print(
        "status=fail; "
        f"languages={','.join(languages)}; "
        f"reason_count={len(failed_reasons)}; "
        f"reasons={' | '.join(failed_reasons)}"
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
