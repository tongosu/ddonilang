#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


def run_token_check(name: str, text_by_label: dict[str, str], required: dict[str, list[str]]) -> dict:
    missing: list[str] = []
    for label, tokens in required.items():
        hay = text_by_label.get(label, "")
        for token in tokens:
            if token not in hay:
                missing.append(f"{label}:{token}")
    return {"name": name, "ok": len(missing) == 0, "missing": missing}


def run_forbidden_token_check(name: str, text_by_label: dict[str, str], forbidden: dict[str, list[str]]) -> dict:
    found: list[str] = []
    for label, tokens in forbidden.items():
        hay = text_by_label.get(label, "")
        for token in tokens:
            if token in hay:
                found.append(f"{label}:{token}")
    return {"name": name, "ok": len(found) == 0, "missing": found}


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate: Seamgrim sim-core minimal contract")
    parser.add_argument("--index-html", default="solutions/seamgrim_ui_mvp/ui/index.html")
    parser.add_argument("--app-js", default="solutions/seamgrim_ui_mvp/ui/app.js")
    parser.add_argument("--browse-js", default="solutions/seamgrim_ui_mvp/ui/screens/browse.js")
    parser.add_argument("--run-js", default="solutions/seamgrim_ui_mvp/ui/screens/run.js")
    parser.add_argument("--dotbogi-js", default="solutions/seamgrim_ui_mvp/ui/components/dotbogi.js")
    parser.add_argument("--styles", default="solutions/seamgrim_ui_mvp/ui/styles.css")
    parser.add_argument("--json-out", default="", help="optional json report path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    paths = {
        "html": root / args.index_html,
        "app": root / args.app_js,
        "browse": root / args.browse_js,
        "run": root / args.run_js,
        "dotbogi": root / args.dotbogi_js,
        "css": root / args.styles,
    }
    for label, path in paths.items():
        if not path.exists():
            print(f"missing file: {label}:{path}")
            return 1

    text_by_label = {label: path.read_text(encoding="utf-8") for label, path in paths.items()}
    checks = [
        run_token_check(
            "sim_core_policy_toggle",
            text_by_label,
            {
                "app": [
                    'const SIM_CORE_POLICY_CLASS = "policy-sim-core";',
                    'readWindowBoolean("SEAMGRIM_SIM_CORE_POLICY", true)',
                    "applySimCorePolicy()",
                ],
            },
        ),
        run_token_check(
            "sim_core_overlay_on_bogae",
            text_by_label,
            {
                "run": [
                    "deriveRunKindAndChannels(",
                    'this.isSimCorePolicyEnabled()',
                    'this.overlay.show()',
                ],
            },
        ),
        run_forbidden_token_check(
            "sim_core_removed_nonessential_dom",
            text_by_label,
            {
                "html": [
                    'class="bogae-zoom-controls"',
                    'id="btn-zoom-in"',
                    'id="btn-zoom-out"',
                    'id="btn-zoom-reset"',
                    'class="statusbar"',
                    'id="run-status"',
                    'id="run-hash"',
                ],
                "css": [
                    ".bogae-zoom-controls",
                    ".statusbar",
                ],
                "run": [
                    'querySelector("#btn-zoom-in")',
                    'querySelector("#btn-zoom-out")',
                    'querySelector("#btn-zoom-reset")',
                    'querySelector("#run-status")',
                    'querySelector("#run-hash")',
                ],
            },
        ),
        run_forbidden_token_check(
            "sim_core_dotbogi_graph_only_ui",
            text_by_label,
            {
                "html": [
                    'id="dotbogi-table"',
                    'id="dotbogi-text"',
                    'class="panel-tabs"',
                    'data-view="table"',
                    'data-view="text"',
                    'id="btn-axis-lock"',
                    'id="btn-graph-reset"',
                    'class="graph-range-controls"',
                    'id="graph-preset-slot"',
                    'id="btn-graph-range-apply"',
                    'id="btn-graph-range-reset"',
                    'id="btn-graph-range-save"',
                    'id="btn-graph-range-load"',
                    'class="bogae-range-controls"',
                    'id="bogae-preset-slot"',
                    'id="btn-bogae-range-apply"',
                    'id="btn-bogae-range-reset"',
                    'id="btn-bogae-range-save"',
                    'id="btn-bogae-range-load"',
                ],
            },
        ),
        run_forbidden_token_check(
            "sim_core_dotbogi_graph_only_logic",
            text_by_label,
            {
                "run": ['this.dotbogi.switchTab("graph")'],
                "dotbogi": ["switchTab(", "renderTable(", "setText("],
            },
        ),
        run_forbidden_token_check(
            "sim_core_runtime_summary_minimal_fields",
            text_by_label,
            {
                "run": [
                    "lastRuntimeSignature",
                    "lastRuntimeStatus",
                    "setStatus(",
                    "lastRunStatus",
                    "lastRunHasSpace2d",
                ],
                "browse": [
                    "lastRunStatus",
                    "lastRunHasSpace2d",
                ],
            },
        ),
        run_token_check(
            "sim_core_minimal_required_ui",
            text_by_label,
            {
                "html": [
                    'id="canvas-bogae"',
                    'id="overlay-description"',
                    'id="select-x-axis"',
                    'id="select-y-axis"',
                    'id="btn-overlay-toggle"',
                ],
            },
        ),
    ]

    failed = [row for row in checks if not row["ok"]]
    payload = {
        "schema": "seamgrim.sim_core_contract_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": len(failed) == 0,
        "checks": checks,
    }
    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"[sim-core-gate] report={out}")

    print(f"[sim-core-gate] ok={int(payload['ok'])} checks={len(checks)} failed={len(failed)}")
    if failed:
        for row in failed:
            first = row["missing"][0] if row["missing"] else "-"
            print(f"check={row['name']} missing={first}")
        print("seamgrim sim core contract gate failed")
        return 1
    print("seamgrim sim core contract gate ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
