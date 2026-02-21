#!/usr/bin/env python3
"""Create audit report for rewrite template coverage and risk heuristics."""

from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from pathlib import Path


def template_expected_match(template_id: str, lesson_id: str) -> bool:
    lid = lesson_id.lower()
    tid = template_id.lower()
    if tid == "physics_orbit_v1":
        return any(k in lid for k in ("orbit", "projectile", "centripetal"))
    if tid == "physics_thermal_v1":
        return any(k in lid for k in ("thermal", "cooling", "heat", "gas", "boiling"))
    if tid == "physics_oscillator_v1":
        return any(k in lid for k in ("harmonic", "damped", "resonance", "pendulum", "wave", "spring"))
    if tid == "math_linear_v1":
        return any(k in lid for k in ("line", "linear", "slope"))
    if tid == "math_matrix_v1":
        return any(k in lid for k in ("matrix", "vector", "det"))
    if tid == "math_sequence_v1":
        return any(k in lid for k in ("series", "sequence", "progression"))
    if tid == "math_stats_v1":
        return any(k in lid for k in ("stats", "probability", "survey", "mean"))
    if tid == "economy_growth_v1":
        return any(k in lid for k in ("growth", "productivity", "population", "timeline"))
    if tid == "economy_budget_v1":
        return any(k in lid for k in ("budget", "saving", "allowance", "unit_price"))
    if tid == "economy_stock_flow_v1":
        return any(k in lid for k in ("stock", "flow", "inventory", "store"))
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--manifest",
        default="solutions/seamgrim_ui_mvp/lessons_rewrite_v1/rewrite_manifest.detjson",
        help="Rewrite manifest path",
    )
    parser.add_argument(
        "--out",
        default="build/reports/seamgrim_rewrite_audit_report_v1.detjson",
        help="Audit output path",
    )
    args = parser.parse_args()

    manifest = json.loads(Path(args.manifest).read_text(encoding="utf-8"))
    rows = list(manifest.get("generated", []))

    by_subject = Counter()
    by_template = Counter()
    template_by_subject = defaultdict(Counter)
    mismatch_rows = []

    for row in rows:
        lesson_id = str(row.get("lesson_id", "")).strip()
        subject = str(row.get("subject", "")).strip()
        template_id = str(row.get("template_id", "")).strip()
        by_subject[subject] += 1
        by_template[template_id] += 1
        template_by_subject[subject][template_id] += 1
        if lesson_id and template_id and not template_expected_match(template_id, lesson_id):
            mismatch_rows.append(
                {
                    "lesson_id": lesson_id,
                    "subject": subject,
                    "template_id": template_id,
                    "reason": "lesson_id keyword vs template mismatch",
                }
            )

    report = {
        "schema": "seamgrim.rewrite.audit_report.v1",
        "manifest": str(Path(args.manifest).as_posix()),
        "count": len(rows),
        "by_subject": dict(by_subject),
        "by_template": dict(by_template),
        "template_by_subject": {k: dict(v) for k, v in template_by_subject.items()},
        "mismatch_count": len(mismatch_rows),
        "mismatches": mismatch_rows,
    }

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[ok] wrote {out_path} mismatch={len(mismatch_rows)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
