#!/usr/bin/env python3
"""Fail-closed drift checker for async observability endpoint marker and docs parity."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    DecisionAccumulator,
    fail,
    write_json,
)

SCHEMA_VERSION = "kamn.ci.observability-endpoint-drift-report.v1"
DOCS_MIGRATION_MARKER = (
    "Runtime observability endpoint ingress runs on async tokio listener "
    "path; drift contracts enforce fail-closed parity for unknown-path, "
    "malformed-request, and timeout compatibility behavior."
)

SOURCE_MARKERS: dict[str, str] = {
    "async_io_import": "use tokio::io::{AsyncReadExt, AsyncWriteExt};",
    "async_listener_bind": "let listener = tokio::net::TcpListener::bind(config.bind_addr.as_str())",
    "async_request_parser": "async fn read_http_request_path_async(",
    "async_response_writer": "async fn write_http_response_async(",
    "async_dispatch": "async fn dispatch_observability_endpoint_request(",
    "async_not_found_handler": "handle_observability_not_found_path().await",
    "async_idle_timeout_reason": "observability endpoint timed out after {} ms waiting for requests",
}

MAIN_WIRING_MARKER = "serve_observability_endpoint(&endpoint_config, &snapshot)"
FRAMEWORK_INGRESS_MARKER = "tokio::net::TcpListener::bind(config.bind_addr.as_str())"


def _read_text(path: Path, *, reason_code: str) -> str:
    if not path.is_file():
        fail(reason_code)
    return path.read_text(encoding="utf-8")


def _run(args: argparse.Namespace) -> int:
    source_file = Path(args.source_file).resolve()
    main_file = Path(args.main_file).resolve()
    framework_file = Path(args.framework_file).resolve()
    docs_file = Path(args.docs_file).resolve()

    source_text = _read_text(
        source_file,
        reason_code="observability_source_file_missing",
    )
    source_contract_text = source_text.split("\n#[cfg(test)]", 1)[0]
    main_text = _read_text(
        main_file,
        reason_code="observability_main_file_missing",
    )
    framework_text = _read_text(
        framework_file,
        reason_code="observability_framework_file_missing",
    )
    docs_text = _read_text(
        docs_file,
        reason_code="observability_docs_file_missing",
    )

    decision = DecisionAccumulator()

    missing_source_markers: list[str] = []
    for marker_key, marker_value in SOURCE_MARKERS.items():
        if marker_value not in source_contract_text:
            missing_source_markers.append(marker_key)
            decision.reject_if(
                True,
                f"observability_source_marker_missing:{marker_key}",
            )

    decision.reject_if(
        MAIN_WIRING_MARKER not in main_text,
        "observability_main_wiring_marker_missing",
    )
    decision.reject_if(
        FRAMEWORK_INGRESS_MARKER not in framework_text,
        "observability_framework_marker_missing",
    )
    decision.reject_if(
        DOCS_MIGRATION_MARKER not in docs_text,
        "observability_docs_marker_missing",
    )

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"

    report = {
        "schema_version": SCHEMA_VERSION,
        "status": status,
        "final_decision": final_decision,
        "observability_async_ingress_contract_status": (
            "verified" if not missing_source_markers else "rejected"
        ),
        "observability_framework_parity_status": (
            "verified"
            if MAIN_WIRING_MARKER in main_text and FRAMEWORK_INGRESS_MARKER in framework_text
            else "rejected"
        ),
        "docs_migration_contract_status": (
            "verified" if DOCS_MIGRATION_MARKER in docs_text else "rejected"
        ),
        "source_marker_count": len(SOURCE_MARKERS) - len(missing_source_markers),
        "expected_source_marker_count": len(SOURCE_MARKERS),
        "reason_codes": reason_codes,
        "source_file": str(source_file),
        "main_file": str(main_file),
        "framework_file": str(framework_file),
        "docs_file": str(docs_file),
        "generated_at_epoch": int(time.time()),
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, report)

    reason_codes_csv = ",".join(reason_codes)
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(
        "observability_async_ingress_contract_status="
        f"{report['observability_async_ingress_contract_status']}"
    )
    print(
        "observability_framework_parity_status="
        f"{report['observability_framework_parity_status']}"
    )
    print(f"docs_migration_contract_status={report['docs_migration_contract_status']}")
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"report_file={output_json}")

    if final_decision != "GO":
        fail(f"observability endpoint drift contract failed: {reason_codes_csv}")

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check fail-closed observability endpoint drift contract markers."
    )
    parser.add_argument(
        "--source-file",
        default=str(ROOT_DIR / "crates/kamn-node/src/observability_endpoint.rs"),
        help="Observability endpoint source path.",
    )
    parser.add_argument(
        "--main-file",
        default=str(ROOT_DIR / "crates/kamn-node/src/main.rs"),
        help="Main runtime wiring source path.",
    )
    parser.add_argument(
        "--framework-file",
        default=str(ROOT_DIR / "crates/kamn-node/src/service_api_endpoint.rs"),
        help="Framework ingress source path for async marker checks.",
    )
    parser.add_argument(
        "--docs-file",
        default=str(ROOT_DIR / "docs/foundation/node-runtime-cli.md"),
        help="Node runtime CLI documentation path.",
    )
    parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for drift report JSON.",
    )

    args = parser.parse_args()
    return _run(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
