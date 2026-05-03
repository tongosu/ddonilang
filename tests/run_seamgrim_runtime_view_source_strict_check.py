#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    run_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
    graph_autorender_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "graph_autorender.js"
    playground_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "playground.js"
    inspector_contract_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "inspector_contract.js"
    exec_server_py = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server.py"

    missing_paths = [path for path in [run_js, graph_autorender_js, playground_js, inspector_contract_js, exec_server_py] if not path.exists()]
    if missing_paths:
        print("check=runtime_view_source_strict detail=missing_paths:" + ",".join(str(path) for path in missing_paths))
        return 1

    run_text = _read_text(run_js)
    graph_text = _read_text(graph_autorender_js)
    playground_text = _read_text(playground_js)
    inspector_text = _read_text(inspector_contract_js)
    exec_server_text = _read_text(exec_server_py)

    required_checks: list[tuple[bool, str]] = [
        ("resolveRuntimeViewSourceStrictness" in run_text, "run_helper_missing"),
        ("보기소스경고" in run_text, "run_hint_missing"),
        (run_text.count("allowObservationOutputFallback: false") >= 3, "run_allow_observation_output_fallback_false_missing"),
        ("allowObservationOutputFallback: false" in graph_text, "graph_autorender_strict_missing"),
        ("allowObservationOutputFallback: false" in playground_text, "playground_strict_missing"),
        ("NON_STRICT_VIEW_SOURCES" in inspector_text, "inspector_non_strict_source_registry_missing"),
        ("보기 엄격성:" in inspector_text, "inspector_view_contract_strict_line_missing"),
        ('"view_contract"' in exec_server_text, "api_run_view_contract_missing"),
        ('"source": "api_run"' in exec_server_text, "api_run_view_contract_source_missing"),
    ]

    failures = [detail for ok, detail in required_checks if not ok]
    if failures:
        print("check=runtime_view_source_strict detail=" + ",".join(failures))
        return 1

    print("seamgrim runtime view source strict check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
