#!/usr/bin/env python
import argparse
import json
import os
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from urllib.parse import urlparse

# Reuse export_graph helpers
TOOLS_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS_DIR))

from export_graph import (
    run_teul_cli,
    parse_points,
    build_graph,
    hash_text,
    extract_meta,
    extract_series_labels,
    normalize_ddn_for_hash,
    preprocess_ddn_for_teul,
    _parse_numbers_from_line,
    normalize_series_label,
)

ROOT = Path(__file__).resolve().parents[3]
UI_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui"
LESSON_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
SEED_LESSON_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1"
REWRITE_LESSON_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons_rewrite_v1"
SCHEMA_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "schema"
SAMPLES_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "samples"
SPACE2D_MARKERS = {"space2d", "2d", "공간", "공간2d"}
SPACE2D_SHAPE_MARKERS = {"space2d.shape", "space2d_shape", "shape2d"}
SPACE2D_SHAPE_KEYS = {
    "x1",
    "y1",
    "x2",
    "y2",
    "cx",
    "cy",
    "r",
    "x",
    "y",
    "size",
    "stroke",
    "fill",
    "color",
    "width",
    "token",
    "id",
    "name",
    "label",
    "토큰",
}
TEXT_MARKERS = {"text", "문서", "해설", "설명", "caption", "자막"}
TEXT_END_MARKERS = {"text.end", "endtext", "문서끝", "끝"}
TEXT_PREFIXES = ("text:", "문서:", "해설:", "설명:", "caption:", "자막:")
TABLE_MARKERS = {"table", "표", "테이블"}
TABLE_END_MARKERS = {"table.end", "endtable", "표끝", "테이블끝"}
STRUCTURE_MARKERS = {"structure", "구조", "그래프구조"}
STRUCTURE_END_MARKERS = {"structure.end", "endstructure", "구조끝"}


def resolve_build_dir() -> Path:
    primary = Path("I:/home/urihanl/ddn/codex/build")
    fallback = Path("C:/ddn/codex/build")
    target = primary if primary.exists() else fallback
    target.mkdir(parents=True, exist_ok=True)
    return target


def json_response(handler: BaseHTTPRequestHandler, status: int, payload: dict):
    body = json.dumps(payload, ensure_ascii=False, sort_keys=True).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json; charset=utf-8")
    handler.send_header("Access-Control-Allow-Origin", "*")
    handler.send_header("Access-Control-Allow-Headers", "Content-Type")
    handler.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def send_file(handler: BaseHTTPRequestHandler, path: Path):
    if not path.exists():
        handler.send_response(404)
        handler.end_headers()
        return
    content = path.read_bytes()
    suffix = path.suffix.lower()
    content_type = "text/plain"
    if suffix == ".html":
        content_type = "text/html; charset=utf-8"
    elif suffix == ".css":
        content_type = "text/css; charset=utf-8"
    elif suffix == ".js":
        content_type = "application/javascript; charset=utf-8"
    elif suffix == ".json":
        content_type = "application/json; charset=utf-8"
    elif suffix == ".detjson":
        content_type = "application/json; charset=utf-8"
    elif suffix == ".wasm":
        content_type = "application/wasm"
    elif suffix == ".csv":
        content_type = "text/csv; charset=utf-8"
    elif suffix == ".toml":
        content_type = "text/plain; charset=utf-8"
    handler.send_response(200)
    handler.send_header("Cache-Control", "no-store, max-age=0")
    handler.send_header("Content-Type", content_type)
    handler.send_header("Access-Control-Allow-Origin", "*")
    handler.send_header("Content-Length", str(len(content)))
    handler.end_headers()
    handler.wfile.write(content)


def parse_space2d_points(lines: list[str]) -> list[dict]:
    points: list[dict] = []
    idx = 0
    while idx < len(lines):
        marker = lines[idx].strip()
        if marker in SPACE2D_MARKERS:
            nums = []
            j = idx + 1
            while j < len(lines) and len(nums) < 2:
                values, mode = _parse_numbers_from_line(lines[j])
                if mode == "single":
                    nums.extend(values)
                elif mode in ("pair", "triple") and not nums:
                    nums.append(values[0])
                    nums.append(values[1])
                j += 1
            if len(nums) == 2:
                points.append({"x": float(nums[0]), "y": float(nums[1])})
            idx = j
            continue
        idx += 1
    return points


def _parse_space2d_shape(lines: list[str], idx: int) -> tuple[dict | None, int]:
    while idx < len(lines) and not lines[idx].strip():
        idx += 1
    if idx >= len(lines):
        return None, idx
    kind = lines[idx].strip().lower()
    idx += 1
    data: dict[str, object] = {"kind": kind}
    while idx < len(lines):
        key = lines[idx].strip()
        if not key:
            idx += 1
            continue
        if key in SPACE2D_MARKERS or key in SPACE2D_SHAPE_MARKERS:
            break
        if key.startswith("series:"):
            break
        if key not in SPACE2D_SHAPE_KEYS:
            break
        if idx + 1 >= len(lines):
            idx += 1
            break
        value_line = lines[idx + 1].strip()
        if key in ("stroke", "fill", "color", "token", "id", "name", "label", "토큰"):
            data[key] = value_line
        else:
            try:
                data[key] = float(value_line)
            except Exception:
                pass
        idx += 2

    if kind in ("line", "선", "segment"):
        if not all(k in data for k in ("x1", "y1", "x2", "y2")):
            return None, idx
        result = {
            "kind": "line",
            "x1": float(data["x1"]),
            "y1": float(data["y1"]),
            "x2": float(data["x2"]),
            "y2": float(data["y2"]),
            "stroke": data.get("stroke"),
            "width": data.get("width"),
        }
        if data.get("token") or data.get("토큰"):
            result["token"] = data.get("token") or data.get("토큰")
        if data.get("id") or data.get("name") or data.get("label"):
            result["id"] = data.get("id") or data.get("name") or data.get("label")
        return result, idx
    if kind in ("circle", "원"):
        cx = data.get("cx", data.get("x"))
        cy = data.get("cy", data.get("y"))
        r = data.get("r")
        if cx is None or cy is None or r is None:
            return None, idx
        result = {
            "kind": "circle",
            "x": float(cx),
            "y": float(cy),
            "r": float(r),
            "stroke": data.get("stroke"),
            "fill": data.get("fill"),
            "width": data.get("width"),
        }
        if data.get("token") or data.get("토큰"):
            result["token"] = data.get("token") or data.get("토큰")
        if data.get("id") or data.get("name") or data.get("label"):
            result["id"] = data.get("id") or data.get("name") or data.get("label")
        return result, idx
    if kind in ("point", "점"):
        if "x" not in data or "y" not in data:
            return None, idx
        result = {
            "kind": "point",
            "x": float(data["x"]),
            "y": float(data["y"]),
            "size": data.get("size"),
            "color": data.get("color"),
            "stroke": data.get("stroke"),
        }
        if data.get("token") or data.get("토큰"):
            result["token"] = data.get("token") or data.get("토큰")
        if data.get("id") or data.get("name") or data.get("label"):
            result["id"] = data.get("id") or data.get("name") or data.get("label")
        return result, idx
    return None, idx


def parse_space2d_shapes(lines: list[str]) -> list[dict]:
    current: list[dict] = []
    latest: list[dict] = []
    idx = 0
    while idx < len(lines):
        marker = lines[idx].strip()
        if marker in SPACE2D_MARKERS:
            if current:
                latest = current
                current = []
            idx += 1
            continue
        if marker in SPACE2D_SHAPE_MARKERS:
            shape, next_idx = _parse_space2d_shape(lines, idx + 1)
            if shape:
                current.append(shape)
            idx = next_idx
            continue
        idx += 1
    if current:
        latest = current
    return latest


def parse_text_blocks(lines: list[str]) -> str:
    blocks: list[str] = []
    idx = 0
    while idx < len(lines):
        raw = lines[idx].rstrip("\n")
        trimmed = raw.strip()
        lower = trimmed.lower()
        if lower in TEXT_MARKERS:
            idx += 1
            buf: list[str] = []
            while idx < len(lines):
                line = lines[idx].rstrip("\n")
                t = line.strip()
                lower_t = t.lower()
                if not t:
                    if buf:
                        break
                    idx += 1
                    continue
                if (
                    lower_t in TEXT_END_MARKERS
                    or lower_t in SPACE2D_MARKERS
                    or lower_t in SPACE2D_SHAPE_MARKERS
                    or lower_t.startswith("series:")
                ):
                    break
                buf.append(line)
                idx += 1
            if buf:
                blocks.append("\n".join(buf).strip())
            continue
        prefix = next((p for p in TEXT_PREFIXES if lower.startswith(p)), None)
        if prefix:
            blocks.append(raw.split(":", 1)[1].strip())
        idx += 1
    return "\n\n".join(blocks).strip()


def parse_table_blocks(lines: list[str]) -> dict | None:
    """DDN stdout에서 테이블 마커를 인식하여 seamgrim.table.v0 구조를 반환한다."""
    idx = 0
    while idx < len(lines):
        trimmed = lines[idx].strip().lower()
        if trimmed not in TABLE_MARKERS:
            idx += 1
            continue
        idx += 1
        # 다음 비공백 줄을 헤더로 사용
        while idx < len(lines) and not lines[idx].strip():
            idx += 1
        if idx >= len(lines):
            return None
        header_line = lines[idx].strip()
        sep = "\t" if "\t" in header_line else ","
        headers = [h.strip() for h in header_line.split(sep)]
        columns = []
        for h in headers:
            columns.append({"key": h, "label": h})
        idx += 1
        rows: list[dict] = []
        while idx < len(lines):
            line = lines[idx].strip()
            lower_line = line.lower()
            if not line or lower_line in TABLE_END_MARKERS:
                idx += 1
                break
            if lower_line in TEXT_MARKERS or lower_line in SPACE2D_MARKERS or lower_line.startswith("series:"):
                break
            cells = [c.strip() for c in line.split(sep)]
            row: dict = {}
            for i, col in enumerate(columns):
                val = cells[i] if i < len(cells) else ""
                try:
                    row[col["key"]] = float(val) if "." in val else int(val)
                except ValueError:
                    row[col["key"]] = val
            rows.append(row)
            idx += 1
        if columns and rows:
            # 숫자 컬럼 타입 추론
            for col in columns:
                key = col["key"]
                if all(isinstance(r.get(key), (int, float)) for r in rows):
                    col["type"] = "number"
                else:
                    col["type"] = "string"
            return {"columns": columns, "rows": rows}
    return None


def parse_structure_blocks(lines: list[str]) -> dict | None:
    """DDN stdout에서 구조 마커를 인식하여 seamgrim.structure.v0 구조를 반환한다.

    형식:
        structure
        node <id> [label]
        edge <from> <to> [label]
        structure.end
    """
    idx = 0
    while idx < len(lines):
        trimmed = lines[idx].strip().lower()
        if trimmed not in STRUCTURE_MARKERS:
            idx += 1
            continue
        idx += 1
        nodes: list[dict] = []
        edges: list[dict] = []
        node_ids: set[str] = set()
        while idx < len(lines):
            line = lines[idx].strip()
            lower_line = line.lower()
            if not line or lower_line in STRUCTURE_END_MARKERS:
                idx += 1
                break
            if lower_line in TEXT_MARKERS or lower_line in SPACE2D_MARKERS or lower_line in TABLE_MARKERS:
                break
            parts = line.split()
            cmd = parts[0].lower()
            if cmd in ("node", "노드") and len(parts) >= 2:
                nid = parts[1]
                label = " ".join(parts[2:]) if len(parts) > 2 else nid
                if nid not in node_ids:
                    nodes.append({"id": nid, "label": label})
                    node_ids.add(nid)
            elif cmd in ("edge", "간선", "연결") and len(parts) >= 3:
                edge: dict = {"from": parts[1], "to": parts[2], "directed": True}
                if len(parts) > 3:
                    edge["label"] = " ".join(parts[3:])
                edges.append(edge)
            idx += 1
        if nodes:
            return {"nodes": nodes, "edges": edges}
    return None


class DdnExecHandler(BaseHTTPRequestHandler):
    def do_OPTIONS(self):
        self.send_response(204)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.end_headers()

    def do_GET(self):
        parsed = urlparse(self.path)
        path = parsed.path
        if path == "/health" or path == "/api/health":
            json_response(self, 200, {"ok": True, "service": "ddn_exec_server"})
            return
        if path == "/" or path == "/index.html":
            send_file(self, UI_DIR / "index.html")
            return
        if path.startswith("/lessons/"):
            lesson_path = (LESSON_DIR / path[len("/lessons/"):]).resolve()
            if LESSON_DIR in lesson_path.parents or lesson_path == LESSON_DIR:
                send_file(self, lesson_path)
                return
        if path.startswith("/seed_lessons_v1/"):
            seed_path = (SEED_LESSON_DIR / path[len("/seed_lessons_v1/"):]).resolve()
            if SEED_LESSON_DIR in seed_path.parents or seed_path == SEED_LESSON_DIR:
                send_file(self, seed_path)
                return
        if path.startswith("/lessons_rewrite_v1/"):
            rewrite_path = (REWRITE_LESSON_DIR / path[len("/lessons_rewrite_v1/"):]).resolve()
            if REWRITE_LESSON_DIR in rewrite_path.parents or rewrite_path == REWRITE_LESSON_DIR:
                send_file(self, rewrite_path)
                return
        if path.startswith("/schema/"):
            schema_path = (SCHEMA_DIR / path[len("/schema/"):]).resolve()
            if SCHEMA_DIR in schema_path.parents or schema_path == SCHEMA_DIR:
                send_file(self, schema_path)
                return
        if path.startswith("/samples/"):
            sample_path = (SAMPLES_DIR / path[len("/samples/"):]).resolve()
            if SAMPLES_DIR in sample_path.parents or sample_path == SAMPLES_DIR:
                send_file(self, sample_path)
                return
        file_path = (UI_DIR / path.lstrip("/")).resolve()
        if UI_DIR in file_path.parents:
            send_file(self, file_path)
            return
        self.send_response(404)
        self.end_headers()

    def do_POST(self):
        parsed = urlparse(self.path)
        if parsed.path != "/api/run":
            json_response(self, 404, {"ok": False, "error": "not found"})
            return
        try:
            length = int(self.headers.get("Content-Length", "0"))
            body = self.rfile.read(length).decode("utf-8")
            payload = json.loads(body)
            ddn_text = payload.get("ddn_text")
            label = payload.get("label")
            if not isinstance(ddn_text, str) or not ddn_text.strip():
                json_response(self, 400, {"ok": False, "error": "ddn_text required"})
                return

            build_dir = resolve_build_dir()
            temp_path = build_dir / f"seamgrim_ui_mvp_input_{os.getpid()}.ddn"
            preprocessed = preprocess_ddn_for_teul(ddn_text, strip_draw=True)
            temp_path.write_text(preprocessed, encoding="utf-8")

            meta = extract_meta(ddn_text)
            series_labels = extract_series_labels(ddn_text)
            source_hash = hash_text(normalize_ddn_for_hash(ddn_text))
            lines = run_teul_cli(ROOT, temp_path)
            points = parse_points(lines, series_labels)
            space2d_points = parse_space2d_points(lines)
            space2d_shapes = parse_space2d_shapes(lines)
            text_block = parse_text_blocks(lines)
            table_data = parse_table_blocks(lines)
            structure_data = parse_structure_blocks(lines)

            if not label:
                label = meta.get("name") or meta.get("desc") or "f(x)"

            graph = build_graph(points, label, source_hash)
            if series_labels:
                graph["meta"]["series_labels"] = series_labels
                graph["series"][0]["id"] = normalize_series_label(series_labels[0])
            if meta.get("name"):
                graph["meta"]["input_name"] = meta["name"]
            if meta.get("desc"):
                graph["meta"]["input_desc"] = meta["desc"]

            payload = {"ok": True, "graph": graph}
            if space2d_points or space2d_shapes:
                payload["space2d"] = {
                    "schema": "seamgrim.space2d.v0",
                    "points": space2d_points,
                    "shapes": space2d_shapes,
                    "meta": {"title": meta.get("name") or "space2d"},
                }
            if text_block:
                payload["text"] = {
                    "schema": "seamgrim.text.v0",
                    "content": text_block,
                    "format": "markdown",
                }
            if table_data:
                payload["table"] = {
                    "schema": "seamgrim.table.v0",
                    "columns": table_data["columns"],
                    "rows": table_data["rows"],
                    "meta": {"title": meta.get("name") or "table"},
                }
            if structure_data:
                payload["structure"] = {
                    "schema": "seamgrim.structure.v0",
                    "nodes": structure_data["nodes"],
                    "edges": structure_data["edges"],
                    "layout": {"type": "dag"},
                    "meta": {"title": meta.get("name") or "structure"},
                }
            json_response(self, 200, payload)
        except Exception as exc:
            json_response(self, 500, {"ok": False, "error": str(exc)})


def parse_args(argv: list[str] | None = None):
    parser = argparse.ArgumentParser(description="Seamgrim DDN exec server")
    parser.add_argument(
        "--host",
        default=os.environ.get("DDN_EXEC_SERVER_HOST", "127.0.0.1"),
        help="bind host (default: env DDN_EXEC_SERVER_HOST or 127.0.0.1)",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=int(os.environ.get("DDN_EXEC_SERVER_PORT", "8787")),
        help="bind port (default: env DDN_EXEC_SERVER_PORT or 8787)",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None):
    args = parse_args(argv)
    host = str(args.host).strip() or "127.0.0.1"
    port = int(args.port)
    if port < 1 or port > 65535:
        raise SystemExit("port must be in 1..65535")
    server = HTTPServer((host, port), DdnExecHandler)
    print(f"Seamgrim ddn exec server running on http://{host}:{port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
