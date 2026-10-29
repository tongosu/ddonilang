#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _node_runner_contract import run_node_runner


ROOT = Path(__file__).resolve().parents[1]
NODE_RUNNER_TIMEOUT_SEC = 120


def fail(detail: str) -> int:
    print(f"check=seamgrim_platform_server_adapter_contract detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    app_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
    server_contract_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "platform_server_adapter_contract.js"
    contract_runner = ROOT / "tests" / "seamgrim_platform_server_adapter_contract_runner.mjs"
    integration_runner = ROOT / "tests" / "seamgrim_platform_server_adapter_integration_runner.mjs"
    required = [app_js, server_contract_js, contract_runner, integration_runner]
    missing = [str(path) for path in required if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))

    app_text = _read(app_js)
    contract_text = _read(server_contract_js)

    static_required = [
        ("from \"./platform_server_adapter_contract.js\"" in app_text, "server_contract_import_missing"),
        ("function resolvePlatformServerAdapterEnabled()" in app_text, "server_adapter_enable_resolver_missing"),
        ("function emitPlatformServerAdapterExchange(request = null, response = null)" in app_text, "server_exchange_helper_missing"),
        ("function maybeEmitPlatformServerAdapterForOp(op, payload = {}, fallbackMessage" in app_text, "server_adapter_emit_helper_missing"),
        ("PlatformServerAdapterOp.SAVE" in app_text, "server_op_save_usage_missing"),
        ("PlatformServerAdapterOp.RESTORE_REVISION" in app_text, "server_op_restore_usage_missing"),
        ("PlatformServerAdapterOp.SHARE" in app_text, "server_op_share_usage_missing"),
        ("PlatformServerAdapterOp.PUBLISH" in app_text, "server_op_publish_usage_missing"),
        ("PlatformServerAdapterOp.INSTALL_PACKAGE" in app_text, "server_op_install_usage_missing"),
        ("PlatformServerAdapterOp.SWITCH_CATALOG" in app_text, "server_op_switch_catalog_usage_missing"),
        ("window.__SEAMGRIM_PLATFORM_SERVER_LAST_REQUEST__" in app_text, "server_request_snapshot_binding_missing"),
        ("window.__SEAMGRIM_PLATFORM_SERVER_LAST_RESPONSE__" in app_text, "server_response_snapshot_binding_missing"),
        (
            "export const PLATFORM_SERVER_ADAPTER_REQUEST_SCHEMA = \"seamgrim.platform.server_adapter.request.v1\""
            in contract_text,
            "server_request_schema_missing",
        ),
        (
            "export const PLATFORM_SERVER_ADAPTER_RESPONSE_SCHEMA = \"seamgrim.platform.server_adapter.response.v1\""
            in contract_text,
            "server_response_schema_missing",
        ),
        ("export const PlatformServerAdapterErrorCode = Object.freeze({" in contract_text, "server_error_code_contract_missing"),
        ("PlatformServerAdapterErrorCode.PERMISSION_DENIED" in contract_text, "server_permission_denied_code_missing"),
        ("PlatformServerAdapterErrorCode.VALIDATION_FAILED" in contract_text, "server_validation_failed_code_missing"),
        ("export const PlatformServerAdapterUiAction = Object.freeze({" in contract_text, "server_ui_action_contract_missing"),
        ("export function mapServerOpToNotReadyErrorCode(op)" in contract_text, "server_error_map_function_missing"),
        ("export function resolveServerErrorActionRail(code)" in contract_text, "server_action_rail_function_missing"),
        ("export function buildServerAdapterRequest" in contract_text, "server_request_builder_missing"),
        ("export function buildServerAdapterErrorResponse" in contract_text, "server_response_builder_missing"),
    ]
    failures = [name for ok, name in static_required if not ok]
    if failures:
        return fail(",".join(failures))

    for runner in (contract_runner, integration_runner):
        ok, elapsed_ms, detail = run_node_runner(root=ROOT, runner=runner, timeout_sec=NODE_RUNNER_TIMEOUT_SEC)
        if not ok:
            return fail(f"node_runner_failed:{runner.name}:{detail}")
        print(f"[server-adapter-runner] {runner.name} elapsed_ms={elapsed_ms}:{detail}")
    print("seamgrim platform server adapter contract check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
