#!/usr/bin/env python
from __future__ import annotations

TEXT_MARKERS = {"text", "문서", "해설", "설명", "caption", "자막"}
TEXT_END_MARKERS = {"text.end", "endtext", "문서끝", "끝"}
TEXT_PREFIXES = ("text:", "문서:", "해설:", "설명:", "caption:", "자막:")
TABLE_MARKERS = {"table", "표", "테이블"}
TABLE_END_MARKERS = {"table.end", "endtable", "표끝", "테이블끝"}
TABLE_ROW_MARKERS = {"table.row", "표행"}
STRUCTURE_MARKERS = {"structure", "구조", "그래프구조"}
STRUCTURE_END_MARKERS = {"structure.end", "endstructure", "구조끝"}
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
    "group_id",
    "group",
    "groupId",
    "그룹",
    "묶음",
}
NUMERIC_LITERAL_RE = r"[+-]?(?:\d+(?:\.\d+)?|\.\d+)(?:e[+-]?\d+)?"


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
                trimmed_line = line.strip()
                lower_line = trimmed_line.lower()
                if not trimmed_line:
                    if buf:
                        break
                    idx += 1
                    continue
                if (
                    lower_line in TEXT_END_MARKERS
                    or lower_line in SPACE2D_MARKERS
                    or lower_line in SPACE2D_SHAPE_MARKERS
                    or lower_line in TABLE_MARKERS
                    or lower_line in TABLE_ROW_MARKERS
                    or lower_line in STRUCTURE_MARKERS
                    or lower_line.startswith("series:")
                ):
                    break
                buf.append(line)
                idx += 1
            if buf:
                blocks.append("\n".join(buf).strip())
            continue
        prefix = next((candidate for candidate in TEXT_PREFIXES if lower.startswith(candidate)), None)
        if prefix:
            blocks.append(raw.split(":", 1)[1].strip())
        idx += 1
    return "\n\n".join(blocks).strip()


def parse_space2d_points(lines: list[str]) -> list[dict]:
    points: list[dict] = []
    idx = 0
    while idx < len(lines):
        marker = lines[idx].strip()
        if marker in SPACE2D_MARKERS:
            numbers = []
            cursor = idx + 1
            while cursor < len(lines) and len(numbers) < 2:
                values, mode = _parse_numbers_from_line(lines[cursor])
                if mode == "single":
                    numbers.extend(values)
                elif mode in ("pair", "triple") and not numbers:
                    numbers.extend(values[:2])
                cursor += 1
            if len(numbers) == 2:
                points.append({"x": float(numbers[0]), "y": float(numbers[1])})
            idx = cursor
            continue
        idx += 1
    return points


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


def parse_table_blocks(lines: list[str]) -> dict | None:
    explicit = parse_explicit_table_blocks(lines)
    if explicit:
        return explicit
    return parse_table_row_blocks(lines)


def parse_explicit_table_blocks(lines: list[str]) -> dict | None:
    idx = 0
    while idx < len(lines):
        marker = lines[idx].strip().lower()
        if marker not in TABLE_MARKERS:
            idx += 1
            continue
        idx += 1
        while idx < len(lines) and not lines[idx].strip():
            idx += 1
        if idx >= len(lines):
            return None
        header_line = lines[idx].strip()
        sep = "\t" if "\t" in header_line else ","
        columns = [{"key": header.strip(), "label": header.strip()} for header in header_line.split(sep)]
        idx += 1
        rows: list[dict] = []
        while idx < len(lines):
            line = lines[idx].strip()
            lower_line = line.lower()
            if not line or lower_line in TABLE_END_MARKERS:
                idx += 1
                break
            if (
                lower_line in TEXT_MARKERS
                or lower_line in STRUCTURE_MARKERS
                or lower_line in SPACE2D_MARKERS
                or lower_line.startswith("series:")
            ):
                break
            cells = [cell.strip() for cell in line.split(sep)]
            row: dict = {}
            for cell_idx, column in enumerate(columns):
                value = cells[cell_idx] if cell_idx < len(cells) else ""
                row[column["key"]] = _coerce_table_cell(value)
            rows.append(row)
            idx += 1
        if columns and rows:
            return finalize_table(columns, rows)
    return None


def parse_table_row_blocks(lines: list[str]) -> dict | None:
    idx = 0
    rows: list[dict] = []
    column_order: list[str] = []
    while idx < len(lines):
        marker = lines[idx].strip().lower()
        if marker not in TABLE_ROW_MARKERS:
            idx += 1
            continue
        idx += 1
        row: dict = {}
        while idx < len(lines):
            key = lines[idx].strip()
            lower_key = key.lower()
            if not key:
                idx += 1
                continue
            if (
                lower_key in TABLE_ROW_MARKERS
                or lower_key in TABLE_MARKERS
                or lower_key in TABLE_END_MARKERS
                or lower_key in TEXT_MARKERS
                or lower_key in STRUCTURE_MARKERS
                or lower_key in STRUCTURE_END_MARKERS
                or lower_key in SPACE2D_MARKERS
                or lower_key in SPACE2D_SHAPE_MARKERS
                or lower_key.startswith("series:")
            ):
                break
            if idx + 1 >= len(lines):
                break
            value_line = lines[idx + 1].strip()
            lower_value = value_line.lower()
            if (
                lower_value in TABLE_ROW_MARKERS
                or lower_value in TABLE_MARKERS
                or lower_value in TABLE_END_MARKERS
                or lower_value in TEXT_MARKERS
                or lower_value in STRUCTURE_MARKERS
                or lower_value in STRUCTURE_END_MARKERS
                or lower_value in SPACE2D_MARKERS
                or lower_value in SPACE2D_SHAPE_MARKERS
                or lower_value.startswith("series:")
            ):
                break
            if key not in column_order:
                column_order.append(key)
            row[key] = _coerce_table_cell(value_line)
            idx += 2
        if row:
            rows.append(row)
            continue
        idx += 1
    if not rows or not column_order:
        return None
    columns = [{"key": key, "label": key} for key in column_order]
    return finalize_table(columns, rows)


def parse_structure_blocks(lines: list[str]) -> dict | None:
    idx = 0
    while idx < len(lines):
        marker = lines[idx].strip().lower()
        if marker not in STRUCTURE_MARKERS:
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
            if (
                lower_line in TEXT_MARKERS
                or lower_line in TABLE_MARKERS
                or lower_line in TABLE_ROW_MARKERS
                or lower_line in SPACE2D_MARKERS
                or lower_line in SPACE2D_SHAPE_MARKERS
            ):
                break
            parts = line.split()
            command = parts[0].lower()
            if command in ("node", "노드") and len(parts) >= 2:
                node_id = parts[1]
                label = " ".join(parts[2:]) if len(parts) > 2 else node_id
                if node_id not in node_ids:
                    nodes.append({"id": node_id, "label": label})
                    node_ids.add(node_id)
            elif command in ("edge", "간선", "연결") and len(parts) >= 3:
                edge: dict = {"from": parts[1], "to": parts[2], "directed": True}
                if len(parts) > 3:
                    edge["label"] = " ".join(parts[3:])
                edges.append(edge)
            idx += 1
        if nodes:
            return {"nodes": nodes, "edges": edges}
    return None


def finalize_table(columns: list[dict], rows: list[dict]) -> dict:
    for column in columns:
        key = column["key"]
        if all(isinstance(row.get(key), (int, float)) for row in rows):
            column["type"] = "number"
        else:
            column["type"] = "string"
    return {"columns": columns, "rows": rows}


def _coerce_table_cell(value: str):
    try:
        return float(value) if "." in value else int(value)
    except ValueError:
        return value


def _parse_numbers_from_line(line: str) -> tuple[list[float], str]:
    parts = [part.strip() for part in str(line or "").split(",")]
    values: list[float] = []
    if len(parts) in (2, 3):
        try:
            for part in parts:
                values.append(float(part))
            return values, "pair" if len(values) == 2 else "triple"
        except Exception:
            values = []
    try:
        return [float(str(line).strip())], "single"
    except Exception:
        return [], "none"


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
        if key in (
            "stroke",
            "fill",
            "color",
            "token",
            "id",
            "name",
            "label",
            "토큰",
            "group_id",
            "group",
            "groupId",
            "그룹",
            "묶음",
        ):
            data[key] = value_line
        else:
            try:
                data[key] = float(value_line)
            except Exception:
                pass
        idx += 2

    if kind in ("line", "선", "segment"):
        if not all(key in data for key in ("x1", "y1", "x2", "y2")):
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
        _copy_space2d_identity_fields(result, data)
        return result, idx
    if kind in ("circle", "원"):
        cx = data.get("cx", data.get("x"))
        cy = data.get("cy", data.get("y"))
        radius = data.get("r")
        if cx is None or cy is None or radius is None:
            return None, idx
        result = {
            "kind": "circle",
            "x": float(cx),
            "y": float(cy),
            "r": float(radius),
            "stroke": data.get("stroke"),
            "fill": data.get("fill"),
            "width": data.get("width"),
        }
        _copy_space2d_identity_fields(result, data)
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
        _copy_space2d_identity_fields(result, data)
        return result, idx
    return None, idx


def _copy_space2d_identity_fields(result: dict, data: dict[str, object]) -> None:
    token = data.get("token") or data.get("토큰")
    if token:
        result["token"] = token
    object_id = data.get("id") or data.get("name") or data.get("label")
    if object_id:
        result["id"] = object_id
    group_id = (
        data.get("group_id")
        or data.get("group")
        or data.get("groupId")
        or data.get("그룹")
        or data.get("묶음")
    )
    if group_id:
        result["group_id"] = group_id
