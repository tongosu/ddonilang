#!/usr/bin/env python
from __future__ import annotations

import importlib.util
from pathlib import Path


def load_module(root: Path):
    path = root / "tests" / "run_seamgrim_ci_gate.py"
    spec = importlib.util.spec_from_file_location("seamgrim_ci_gate", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    mod = load_module(root)

    full_diag = mod.extract_diagnostics(
        "full_check",
        "graph export failed for pack/abc: E_PARSE_UNEXPECTED_TOKEN\nother",
        "",
        False,
    )
    if not full_diag or full_diag[0].get("kind") != "graph_export_failed":
        print("diagnostics check failed: full_check graph_export_failed")
        return 1

    schema_diag = mod.extract_diagnostics(
        "schema_gate",
        "schema_status.json drift detected. regenerate with lesson_schema_upgrade.py --status-out ...",
        "",
        False,
    )
    if not schema_diag or schema_diag[0].get("kind") != "schema_status_drift":
        print("diagnostics check failed: schema_gate schema_status_drift")
        return 1

    ui_diag = mod.extract_diagnostics(
        "ui_age3_gate",
        "check=r3c_media_export missing=html:id=\"media-export-format\"",
        "",
        False,
    )
    if not ui_diag or ui_diag[0].get("kind") != "age3_feature_missing":
        print("diagnostics check failed: ui_age3_gate age3_feature_missing")
        return 1

    space2d_diag = mod.extract_diagnostics(
        "space2d_source_ui_gate",
        "check=playground_space2d_source_persistence missing=html:id=\"space2d-source-mode\"",
        "",
        False,
    )
    if not space2d_diag or space2d_diag[0].get("kind") != "space2d_feature_missing":
        print("diagnostics check failed: space2d_source_ui_gate space2d_feature_missing")
        return 1

    generic_diag = mod.extract_diagnostics("unknown", "line1\nline2", "", False)
    if not generic_diag or generic_diag[0].get("kind") != "generic_error":
        print("diagnostics check failed: generic_error fallback")
        return 1

    print("seamgrim ci gate diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
