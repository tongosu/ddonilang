#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _node_runner_contract import run_node_runner


ROOT = Path(__file__).resolve().parents[1]
NODE_RUNNER_TIMEOUT_SEC = 120


def fail(detail: str) -> int:
    print(f"check=seamgrim_platform_server_action_rail detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    app_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
    run_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
    index_html = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
    server_contract_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "platform_server_adapter_contract.js"
    warning_panel_contract_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "run_warning_panel_contract.js"
    action_rail_contract_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "run_action_rail_contract.js"
    runner = ROOT / "tests" / "seamgrim_platform_server_action_rail_runner.mjs"
    missing = [
        str(path)
        for path in (
            app_js,
            run_js,
            index_html,
            server_contract_js,
            warning_panel_contract_js,
            action_rail_contract_js,
            runner,
        )
        if not path.exists()
    ]
    if missing:
        return fail("missing:" + ",".join(missing))

    app_text = _read(app_js)
    run_text = _read(run_js)
    index_text = _read(index_html)
    contract_text = _read(server_contract_js)
    warning_panel_text = _read(warning_panel_contract_js)
    action_rail_text = _read(action_rail_contract_js)
    static_required = [
        ("resolveServerErrorActionRail" in app_text, "app_action_rail_resolver_usage_missing"),
        ("__SEAMGRIM_PLATFORM_SERVER_LAST_ACTION_RAIL__" in app_text, "app_action_rail_snapshot_binding_missing"),
        ("seamgrim:platform-server-adapter-exchange" in app_text, "app_exchange_event_missing"),
        ("window.dispatchEvent" in app_text, "app_exchange_event_dispatch_missing"),
        ("PLATFORM_UI_ACTION_EVENT" in app_text, "app_platform_ui_action_event_token_missing"),
        ("window.addEventListener(PLATFORM_UI_ACTION_EVENT" in app_text, "app_platform_ui_action_listener_missing"),
        ("handlePlatformUiActionRequest" in app_text, "app_platform_ui_action_handler_missing"),
        ("saveCurrentWork(\"local\"" in app_text, "app_platform_ui_action_local_save_missing"),
        ("applyPlatformServerAdapterExchange" in run_text, "run_exchange_handler_missing"),
        ("PLATFORM_SERVER_EXCHANGE_EVENT" in run_text, "run_exchange_event_token_missing"),
        ("window.addEventListener(PLATFORM_SERVER_EXCHANGE_EVENT" in run_text, "run_exchange_event_listener_missing"),
        ("PLATFORM_UI_ACTION_EVENT" in run_text, "run_platform_ui_action_event_token_missing"),
        ("emitPlatformUiAction" in run_text, "run_platform_ui_action_emit_helper_missing"),
        ("btn-run-warning-platform-login" in run_text, "run_platform_login_button_binding_missing"),
        ("btn-run-warning-platform-request-access" in run_text, "run_platform_request_access_button_binding_missing"),
        ("btn-run-warning-platform-open-local-save" in run_text, "run_platform_open_local_save_button_binding_missing"),
        ("btn-run-warning-platform-login" in index_text, "index_platform_login_button_missing"),
        ("btn-run-warning-platform-request-access" in index_text, "index_platform_request_access_button_missing"),
        ("btn-run-warning-platform-open-local-save" in index_text, "index_platform_open_local_save_button_missing"),
        ("platformErrorCode" in warning_panel_text, "warning_panel_platform_error_contract_missing"),
        ("platformActionRail" in warning_panel_text, "warning_panel_platform_action_rail_contract_missing"),
        ("platformLoginAction" in warning_panel_text, "warning_panel_platform_login_action_missing"),
        ("platformRequestAccessAction" in warning_panel_text, "warning_panel_platform_request_access_action_missing"),
        ("platformOpenLocalSaveAction" in warning_panel_text, "warning_panel_platform_open_local_save_action_missing"),
        ("platformErrorCode" in action_rail_text, "action_rail_platform_error_contract_missing"),
        ("platformActionRail" in action_rail_text, "action_rail_platform_action_rail_contract_missing"),
        ("PlatformServerAdapterErrorCode.PERMISSION_DENIED" in contract_text, "permission_denied_code_missing"),
        ("PlatformServerAdapterErrorCode.VALIDATION_FAILED" in contract_text, "validation_failed_code_missing"),
        ("export const PlatformServerAdapterUiAction = Object.freeze({" in contract_text, "ui_action_contract_missing"),
        ("export function resolveServerErrorActionRail(code)" in contract_text, "action_rail_resolver_missing"),
    ]
    failures = [name for ok, name in static_required if not ok]
    if failures:
        return fail(",".join(failures))

    ok, elapsed_ms, detail = run_node_runner(root=ROOT, runner=runner, timeout_sec=NODE_RUNNER_TIMEOUT_SEC)
    if not ok:
        return fail(f"node_runner_failed:{detail}")

    print(f"[server-action-rail-runner] {runner.name} elapsed_ms={elapsed_ms}:{detail}")
    print("seamgrim platform server action rail check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
