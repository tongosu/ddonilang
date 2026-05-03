#!/usr/bin/env python
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

from _node_runner_contract import run_node_runner


ROOT = Path(__file__).resolve().parents[1]
NODE_RUNNER_TIMEOUT_SEC = 120


def fail(detail: str) -> int:
    print(f"check=seamgrim_platform_mock_interface_contract detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _run_node_runner(path: Path) -> tuple[bool, str]:
    ok, elapsed_ms, detail = run_node_runner(root=ROOT, runner=path, timeout_sec=NODE_RUNNER_TIMEOUT_SEC)
    return ok, f"elapsed_ms={elapsed_ms}:{detail}"


def main() -> int:
    app_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
    contract_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "platform_contract.js"
    adapter_contract_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "platform_mock_adapter_contract.js"
    node_runners = [
        ROOT / "tests" / "seamgrim_auth_save_surface_runner.mjs",
        ROOT / "tests" / "seamgrim_object_revision_surface_runner.mjs",
        ROOT / "tests" / "seamgrim_sharing_publishing_surface_runner.mjs",
        ROOT / "tests" / "seamgrim_package_registry_surface_runner.mjs",
        ROOT / "tests" / "seamgrim_platform_mock_menu_mode_runner.mjs",
    ]

    required_paths = [app_js, contract_js, adapter_contract_js, *node_runners]
    missing = [str(path) for path in required_paths if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))

    app_text = _read(app_js)
    contract_text = _read(contract_js)
    adapter_text = _read(adapter_contract_js)

    static_required = [
        ("function emitPlatformMockAdapterPayload(payload)" in app_text, "emit_helper_missing"),
        ("window.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__ = payload;" in app_text, "emit_window_binding_missing"),
        ("from \"./platform_contract.js\"" in app_text, "platform_contract_import_missing"),
        ("from \"./platform_mock_adapter_contract.js\"" in app_text, "adapter_contract_import_missing"),
        ("function switchCatalog(kind)" in app_text, "switch_catalog_surface_missing"),
        ("function resolvePlatformMockMenuEnabled()" in app_text, "platform_mock_menu_enable_resolver_missing"),
        ("function applyPlatformMockMenuState(buttons = [], enabled = false)" in app_text, "platform_mock_menu_state_helper_missing"),
        ("function saveCurrentWork(target = \"local\"" in app_text, "save_surface_missing"),
        ("function restoreRevision(revisionId = \"\")" in app_text, "restore_surface_missing"),
        ("function shareCurrent(kind = ShareKind.LINK)" in app_text, "share_surface_missing"),
        ("function publishCurrent()" in app_text, "publish_surface_missing"),
        ("function installPackage(packageId = \"\", version = \"\")" in app_text, "install_surface_missing"),
        ("export const RevisionPolicy = Object.freeze({" in contract_text, "revision_policy_missing"),
        ("SOURCE_REVISION_ID_REQUIRED: true" in contract_text, "revision_policy_required_missing"),
        ("export const PublishPolicy = Object.freeze({" in contract_text, "publish_policy_missing"),
        ("ARTIFACT_TRACKS_DRAFT: false" in contract_text, "publish_policy_draft_guard_missing"),
        ("export const CatalogKind = Object.freeze({" in contract_text, "catalog_kind_missing"),
        ("export const ShareKind = Object.freeze({" in contract_text, "share_kind_missing"),
        ("export const ObjectKind = Object.freeze({" in contract_text, "object_kind_missing"),
        ("export const PLATFORM_MOCK_ADAPTER_SCHEMA" in adapter_text, "adapter_schema_missing"),
        ("export function buildMockSaveAdapterPayload" in adapter_text, "adapter_builder_save_missing"),
        ("export function buildMockRestoreRevisionPayload" in adapter_text, "adapter_builder_restore_missing"),
        ("export function buildMockShareAdapterPayload" in adapter_text, "adapter_builder_share_missing"),
        ("export function buildMockPublishAdapterPayload" in adapter_text, "adapter_builder_publish_missing"),
        ("export function buildMockInstallPackagePayload" in adapter_text, "adapter_builder_install_missing"),
        ("export function buildMockSwitchCatalogPayload" in adapter_text, "adapter_builder_switch_missing"),
    ]
    static_failures = [name for ok, name in static_required if not ok]
    if static_failures:
        return fail(",".join(static_failures))

    results: dict[Path, tuple[bool, str]] = {}
    max_workers = max(1, min(4, len(node_runners)))
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        future_to_runner = {executor.submit(_run_node_runner, runner): runner for runner in node_runners}
        for future in as_completed(future_to_runner):
            runner = future_to_runner[future]
            try:
                results[runner] = future.result()
            except Exception as exc:  # pragma: no cover - defensive
                results[runner] = (False, f"internal_runner_exception:{type(exc).__name__}:{exc}")

    for runner in node_runners:
        ok, detail = results.get(runner, (False, "missing_result"))
        if not ok:
            return fail(f"runner_failed:{runner.name}:{detail}")
        print(f"[interface-runner] {runner.name} {detail}")

    print("seamgrim platform mock interface contract check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
