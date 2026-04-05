#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path

from export_graph import compute_result_hash, hash_text, normalize_ddn_for_hash


def canonical_json(value: object) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def write_json(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_json(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8-sig"))
    if not isinstance(data, dict):
        raise ValueError(f"json object required: {path}")
    return data


def extract_points(graph: dict) -> list[dict]:
    series = graph.get("series")
    if not isinstance(series, list) or not series:
        raise ValueError("graph.series missing")
    head = series[0]
    if not isinstance(head, dict):
        raise ValueError("graph.series[0] invalid")
    points = head.get("points")
    if not isinstance(points, list):
        raise ValueError("graph.series[0].points missing")
    normalized: list[dict] = []
    for idx, row in enumerate(points):
        if not isinstance(row, dict):
            raise ValueError(f"point[{idx}] invalid")
        try:
            x = float(row.get("x"))
            y = float(row.get("y"))
        except Exception as exc:  # pragma: no cover - defensive parse branch
            raise ValueError(f"point[{idx}] non-numeric x/y") from exc
        normalized.append({"x": x, "y": y})
    return normalized


def resolve_input_text(
    *,
    input_ddn_path: Path | None,
    snapshot_doc: dict | None,
) -> str:
    if input_ddn_path is not None:
        return input_ddn_path.read_text(encoding="utf-8-sig")
    if snapshot_doc:
        run = snapshot_doc.get("run")
        if isinstance(run, dict):
            source = run.get("source")
            if isinstance(source, dict):
                text = source.get("text")
                if isinstance(text, str) and text.strip():
                    return text
    return ""


def build_bridge_report(
    *,
    graph_doc: dict | None,
    snapshot_doc: dict | None = None,
    input_text: str = "",
    allow_missing_input_text: bool = False,
) -> dict:
    errors: list[str] = []

    snapshot_graph: dict | None = None
    if isinstance(snapshot_doc, dict):
        run = snapshot_doc.get("run")
        if isinstance(run, dict):
            candidate = run.get("graph")
            if isinstance(candidate, dict):
                snapshot_graph = candidate

    active_graph = graph_doc if isinstance(graph_doc, dict) else snapshot_graph
    if not isinstance(active_graph, dict):
        errors.append("graph document unavailable: provide graph or snapshot.run.graph")
        return {
            "schema": "ddn.seamgrim_bridge_check.v1",
            "ok": False,
            "graph_schema_ok": False,
            "snapshot_schema_ok": bool(snapshot_doc is None),
            "graph_snapshot_match": False,
            "input_hash_expected_available": bool(str(input_text).strip()),
            "input_hash_match": None,
            "result_hash_match": False,
            "snapshot_hash_input_match": snapshot_doc is None,
            "snapshot_hash_result_match": snapshot_doc is None,
            "actual_input_hash": "",
            "expected_input_hash": "",
            "actual_result_hash": "",
            "expected_result_hash": "",
            "snapshot_input_hash": "",
            "snapshot_result_hash": "",
            "error_count": len(errors),
            "errors": errors,
        }

    graph_meta = active_graph.get("meta") if isinstance(active_graph.get("meta"), dict) else {}
    actual_input_hash = str(graph_meta.get("source_input_hash", "")).strip()
    actual_result_hash = str(graph_meta.get("result_hash", "")).strip()

    graph_schema_ok = str(active_graph.get("schema", "")).strip() == "seamgrim.graph.v0"
    if not graph_schema_ok:
        errors.append("graph.schema != seamgrim.graph.v0")

    snapshot_schema_ok = True
    if snapshot_doc is not None:
        snapshot_schema_ok = str(snapshot_doc.get("schema", "")).strip() == "seamgrim.snapshot.v0"
        if not snapshot_schema_ok:
            errors.append("snapshot.schema != seamgrim.snapshot.v0")

    graph_snapshot_match = True
    if snapshot_graph is not None:
        graph_snapshot_match = canonical_json(active_graph) == canonical_json(snapshot_graph)
        if not graph_snapshot_match:
            errors.append("graph != snapshot.run.graph")

    expected_result_hash = ""
    result_hash_match = False
    try:
        points = extract_points(active_graph)
        expected_result_hash = f"sha256:{compute_result_hash(points)}"
        result_hash_match = expected_result_hash == actual_result_hash
        if not result_hash_match:
            errors.append("result_hash mismatch")
    except Exception as exc:
        errors.append(f"graph points parse failed: {exc}")

    effective_input_text = str(input_text or "")
    if not effective_input_text.strip() and isinstance(snapshot_doc, dict):
        run = snapshot_doc.get("run")
        if isinstance(run, dict):
            source = run.get("source")
            if isinstance(source, dict):
                maybe_text = source.get("text")
                if isinstance(maybe_text, str):
                    effective_input_text = maybe_text

    input_hash_expected_available = bool(effective_input_text.strip())
    expected_input_hash = ""
    input_hash_match = False
    if input_hash_expected_available:
        expected_input_hash = f"sha256:{hash_text(normalize_ddn_for_hash(effective_input_text))}"
        input_hash_match = expected_input_hash == actual_input_hash
        if not input_hash_match:
            errors.append("source_input_hash mismatch")
    elif not allow_missing_input_text:
        errors.append("source input text unavailable (--input-ddn or snapshot.run.source.text required)")

    snapshot_hash_input_match = True
    snapshot_hash_result_match = True
    snapshot_hash_input = ""
    snapshot_hash_result = ""
    if snapshot_doc is not None:
        run = snapshot_doc.get("run") if isinstance(snapshot_doc, dict) else None
        if not isinstance(run, dict):
            errors.append("snapshot.run missing")
            snapshot_hash_input_match = False
            snapshot_hash_result_match = False
        else:
            hash_obj = run.get("hash")
            if not isinstance(hash_obj, dict):
                errors.append("snapshot.run.hash missing")
                snapshot_hash_input_match = False
                snapshot_hash_result_match = False
            else:
                snapshot_hash_input = str(hash_obj.get("input", "")).strip()
                snapshot_hash_result = str(hash_obj.get("result", "")).strip()
                snapshot_hash_input_match = snapshot_hash_input == actual_input_hash
                snapshot_hash_result_match = snapshot_hash_result == actual_result_hash
                if not snapshot_hash_input_match:
                    errors.append("snapshot.run.hash.input mismatch")
                if not snapshot_hash_result_match:
                    errors.append("snapshot.run.hash.result mismatch")

    return {
        "schema": "ddn.seamgrim_bridge_check.v1",
        "ok": not errors,
        "graph_schema_ok": graph_schema_ok,
        "snapshot_schema_ok": snapshot_schema_ok,
        "graph_snapshot_match": graph_snapshot_match,
        "input_hash_expected_available": input_hash_expected_available,
        "input_hash_match": input_hash_match if input_hash_expected_available else None,
        "result_hash_match": result_hash_match,
        "snapshot_hash_input_match": snapshot_hash_input_match,
        "snapshot_hash_result_match": snapshot_hash_result_match,
        "actual_input_hash": actual_input_hash,
        "expected_input_hash": expected_input_hash,
        "actual_result_hash": actual_result_hash,
        "expected_result_hash": expected_result_hash,
        "snapshot_input_hash": snapshot_hash_input,
        "snapshot_result_hash": snapshot_hash_result,
        "error_count": len(errors),
        "errors": errors,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Check seamgrim.graph.v0/snapshot.v0 hash bridge consistency")
    parser.add_argument("--graph", help="path to seamgrim.graph.v0 json")
    parser.add_argument("--snapshot", help="path to seamgrim.snapshot.v0 json")
    parser.add_argument("--input-ddn", help="optional input ddn path for source_input_hash recomputation")
    parser.add_argument(
        "--allow-missing-input-text",
        action="store_true",
        help="do not fail when input text is unavailable for source_input_hash recomputation",
    )
    parser.add_argument("--out", help="optional report json path")
    args = parser.parse_args()

    graph_path = Path(args.graph).resolve() if args.graph else None
    snapshot_path = Path(args.snapshot).resolve() if args.snapshot else None
    input_ddn_path = Path(args.input_ddn).resolve() if args.input_ddn else None

    if graph_path is None and snapshot_path is None:
        raise ValueError("at least one of --graph/--snapshot is required")

    graph_doc: dict | None = None
    snapshot_doc: dict | None = None
    if graph_path is not None:
        graph_doc = load_json(graph_path)
    if snapshot_path is not None:
        snapshot_doc = load_json(snapshot_path)
    input_text = resolve_input_text(input_ddn_path=input_ddn_path, snapshot_doc=snapshot_doc)
    report = build_bridge_report(
        graph_doc=graph_doc,
        snapshot_doc=snapshot_doc,
        input_text=input_text,
        allow_missing_input_text=bool(args.allow_missing_input_text),
    )

    if args.out:
        write_json(Path(args.out).resolve(), report)

    if report["ok"]:
        print(
            "bridge_check ok "
            f"result_hash={report['result_hash_match']} input_hash={report['input_hash_match']!s}"
        )
        return 0
    print("bridge_check fail " + "; ".join(str(row) for row in report.get("errors", [])))
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
