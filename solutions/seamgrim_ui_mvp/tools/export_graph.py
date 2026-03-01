#!/usr/bin/env python
import argparse
import atexit
import json
import os
import shutil
import subprocess
from pathlib import Path
import hashlib
import re
import tempfile
from datetime import datetime, timezone
from decimal import Decimal, ROUND_HALF_UP
from typing import Optional


def hash_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def normalize_newlines(text: str) -> str:
    return text.replace("\r\n", "\n").replace("\r", "\n")


def split_meta_header(input_text: str) -> tuple[dict, str]:
    meta: dict[str, str] = {}
    lines = normalize_newlines(input_text).split("\n")
    idx = 0
    while idx < len(lines):
        raw = lines[idx]
        trimmed = raw.lstrip(" \t\uFEFF")
        if not trimmed:
            idx += 1
            continue
        if trimmed.startswith("#") and ":" in trimmed:
            key, value = trimmed[1:].split(":", 1)
            key = key.strip()
            if not key:
                break
            meta[key] = value.strip()
            idx += 1
            continue
        break
    body = "\n".join(lines[idx:])
    return meta, body


def normalize_ddn_for_hash(input_text: str) -> str:
    _, body = split_meta_header(input_text)
    return body.lstrip("\n")


def to_decimal(value: object) -> Decimal:
    if isinstance(value, Decimal):
        return value
    return Decimal(str(value))


def quantize_number(value: Decimal, digits: int = 4) -> Decimal:
    step = Decimal("1").scaleb(-digits)
    return value.quantize(step, rounding=ROUND_HALF_UP)


def normalize_points(points: list[dict], digits: int = 4) -> list[dict]:
    out: list[dict] = []
    for point in points:
        x_raw = to_decimal(point["x"])
        y_raw = to_decimal(point["y"])
        x_val = quantize_number(x_raw, digits)
        y_val = quantize_number(y_raw, digits)
        out.append({"x": float(x_val), "y": float(y_val)})
    return out


def canonical_json(data: object) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def compute_result_hash(points: list[dict]) -> str:
    normalized = normalize_points(points, digits=4)
    return hash_text(canonical_json(normalized))


def _normalize_bin_path(project_root: Path, raw_path: str) -> Path:
    path = Path(raw_path.strip())
    if not path.is_absolute():
        path = project_root / path
    return path.resolve()


def _read_teul_cli_bin_from_project(project_root: Path) -> Optional[Path]:
    meta_path = project_root / "ddn.project.json"
    if not meta_path.exists():
        return None
    try:
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
    except Exception:
        return None
    toolchain = meta.get("toolchain")
    if not isinstance(toolchain, dict):
        return None
    value = toolchain.get("teul_cli_bin")
    if not isinstance(value, str) or not value.strip():
        return None
    return _normalize_bin_path(project_root, value)


def _resolve_teul_cli_bin(project_root: Path) -> Path:
    env_path = os.environ.get("TEUL_CLI_BIN")
    if env_path:
        candidate = _normalize_bin_path(project_root, env_path)
        if candidate.exists():
            return candidate

    project_path = _read_teul_cli_bin_from_project(project_root)
    if project_path and project_path.exists():
        return project_path

    mode = os.environ.get("TEUL_CLI_MODE", "").lower()
    suffix = ".exe" if os.name == "nt" else ""
    if mode in ("debug", "release"):
        modes = [mode]
    else:
        modes = ["debug", "release"]
    for build_mode in modes:
        candidate = project_root / "target" / build_mode / f"teul-cli{suffix}"
        if candidate.exists():
            return candidate.resolve()

    path_bin = shutil.which("teul-cli")
    if path_bin:
        return Path(path_bin).resolve()

    raise RuntimeError(
        "E_TEUL_BIN_NOT_FOUND teul-cli 바이너리를 찾을 수 없습니다. "
        "TEUL_CLI_BIN 또는 ddn.project.json.toolchain.teul_cli_bin을 설정하세요."
    )

_worker_client = None
_worker_counter = 1


class _WorkerClient:
    def __init__(self, root: Path):
        self.root = root
        self.teul_cli = _resolve_teul_cli_bin(root)
        self.proc = subprocess.Popen(
            [str(self.teul_cli), "worker"],
            cwd=root,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

    def close(self):
        if self.proc and self.proc.poll() is None:
            self.proc.terminate()

    def request(self, payload: dict) -> dict:
        if self.proc.poll() is not None:
            raise RuntimeError("worker process exited")
        raw = json.dumps(payload, ensure_ascii=False, sort_keys=True).encode("utf-8")
        header = f"Content-Length: {len(raw)}\r\n\r\n".encode("utf-8")
        assert self.proc.stdin is not None
        assert self.proc.stdout is not None
        self.proc.stdin.write(header + raw)
        self.proc.stdin.flush()
        resp = self._read_frame(self.proc.stdout)
        return json.loads(resp.decode("utf-8"))

    def _read_frame(self, stream) -> bytes:
        length = None
        while True:
            line = stream.readline()
            if not line:
                raise RuntimeError("worker response EOF")
            line = line.strip()
            if not line:
                break
            text = line.decode("utf-8", errors="replace").lower()
            if text.startswith("content-length:"):
                length = int(text.split(":", 1)[1].strip())
        if length is None:
            raise RuntimeError("worker response missing content-length")
        body = stream.read(length)
        return body

    def run_file(self, input_path: Path, args: Optional[list[str]] = None) -> list[str]:
        global _worker_counter
        payload = {
            "jsonrpc": "2.0",
            "id": _worker_counter,
            "method": "run_file",
            "params": {
                "path": str(input_path),
                "args": args or [],
                "mode": "inproc",
            },
        }
        _worker_counter += 1
        response = self.request(payload)
        if "error" in response:
            message = response["error"].get("message", "worker error")
            raise RuntimeError(message)
        result = response.get("result", {})
        if not result.get("ok", False):
            stderr_lines = result.get("stderr", [])
            message = "\n".join(stderr_lines) if stderr_lines else "worker run failed"
            raise RuntimeError(message)
        stdout_lines = result.get("stdout", [])
        return [line.strip() for line in stdout_lines if line.strip()]


def _get_worker_client(root: Path) -> _WorkerClient:
    global _worker_client
    if _worker_client is None:
        _worker_client = _WorkerClient(root)
        atexit.register(_worker_client.close)
    return _worker_client


def run_teul_cli(root: Path, input_path: Path, madi: int | None = None) -> list[str]:
    use_worker = os.environ.get("TEUL_CLI_WORKER", "").strip().lower() in ("1", "true", "yes")
    requested_madi = None
    if madi is not None:
        try:
            parsed_madi = int(madi)
            if parsed_madi > 0:
                requested_madi = parsed_madi
        except Exception:
            requested_madi = None

    if use_worker and requested_madi is None:
        client = _get_worker_client(root)
        return client.run_file(input_path)

    teul_cli = _resolve_teul_cli_bin(root)
    cmd = [str(teul_cli), "run", str(input_path)]
    if requested_madi is not None:
        cmd.extend(["--madi", str(requested_madi)])
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or f"teul-cli failed: {result.returncode}")

    lines = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    lines = [
        line
        for line in lines
        if not line.startswith("state_hash=")
        and not line.startswith("trace_hash=")
        and not line.startswith("bogae_hash=")
    ]
    return lines


def _parse_numbers_from_line(line: str) -> tuple[list[Decimal], str]:
    cleaned = line.strip()
    if not cleaned:
        return ([], "empty")
    if "=" in cleaned:
        _, cleaned = cleaned.split("=", 1)
        cleaned = cleaned.strip()
    parts = [p for p in re.split(r"[,\\s]+", cleaned) if p]
    if len(parts) == 2:
        try:
            return ([Decimal(parts[0]), Decimal(parts[1])], "pair")
        except Exception:
            return ([], "skip")
    if len(parts) == 3:
        try:
            return ([Decimal(parts[0]), Decimal(parts[1]), Decimal(parts[2])], "triple")
        except Exception:
            return ([], "skip")
    if len(parts) != 1:
        return ([], "skip")
    try:
        return ([Decimal(parts[0])], "single")
    except Exception as exc:
        return ([], "skip")


def parse_points(lines: list[str], series_labels: Optional[list[str]] = None) -> list[dict]:
    if series_labels:
        label_set = set(series_labels)
        points: list[dict] = []
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            if line in label_set:
                nums: list[Decimal] = []
                j = i + 1
                while j < len(lines) and len(nums) < 2:
                    value, mode = _parse_numbers_from_line(lines[j])
                    if mode == "single":
                        nums.extend(value)
                    elif mode in ("pair", "triple") and not nums:
                        nums.append(value[0])
                        nums.append(value[1])
                    j += 1
                if len(nums) == 2:
                    points.append({"x": nums[0], "y": nums[1]})
                i = j
                continue
            i += 1
        if points:
            return points
    space2d_markers = {"space2d", "2d", "공간", "공간2d"}
    shape_markers = {"space2d.shape", "space2d_shape", "shape2d"}
    shape_keys = {
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
    skip_next_numeric = False
    values: list[Decimal] = []
    points: list[dict] = []
    for line in lines:
        trimmed = line.strip()
        if trimmed in space2d_markers or trimmed in shape_markers:
            skip_next_numeric = False
            continue
        if trimmed in shape_keys:
            skip_next_numeric = True
            continue
        if skip_next_numeric:
            skip_next_numeric = False
            continue
        nums, mode = _parse_numbers_from_line(line)
        if mode in ("empty", "skip"):
            continue
        if mode in ("pair", "triple"):
            if values:
                raise ValueError("mixed output: pair line after single values")
            points.append({"x": nums[0], "y": nums[1]})
        else:
            if points:
                raise ValueError("mixed output: single value after pair lines")
            values.extend(nums)
    if points:
        return points
    if len(values) % 2 != 0:
        raise ValueError("odd number of values; expected x,y pairs")
    return [{"x": values[i], "y": values[i + 1]} for i in range(0, len(values), 2)]


def build_graph(points: list[dict], label: str, source_hash: str) -> dict:
    normalized = normalize_points(points, digits=4)
    xs = [p["x"] for p in normalized]
    ys = [p["y"] for p in normalized]
    result_hash = compute_result_hash(points)
    return {
        "schema": "seamgrim.graph.v0",
        "series": [{"id": "main", "label": label, "points": normalized}],
        "axis": {
            "x_min": min(xs) if xs else 0,
            "x_max": max(xs) if xs else 0,
            "y_min": min(ys) if ys else 0,
            "y_max": max(ys) if ys else 0,
        },
        "meta": {
            "source_input_hash": f"sha256:{source_hash}",
            "result_hash": f"sha256:{result_hash}",
            "source_mode": "teul-cli-stdout",
        },
    }


def extract_meta(input_text: str) -> dict:
    meta, _ = split_meta_header(input_text)
    return {"name": meta.get("이름"), "desc": meta.get("설명")}


def extract_series_labels(input_text: str) -> list[str]:
    labels: list[str] = []
    for match in re.finditer(r"무리\s*<-\s*\"([^\"]+)\"", input_text):
        labels.append(match.group(1))
    for match in re.finditer(r"무리\s*<-\s*'([^']+)'", input_text):
        labels.append(match.group(1))
    return list(dict.fromkeys(labels))


def normalize_series_label(label: str) -> str:
    if label.startswith("series:"):
        return label[len("series:") :]
    return label


def strip_bogae_scene_blocks(input_text: str) -> tuple[str, int]:
    removed = 0
    idx = 0
    out: list[str] = []
    pattern = re.compile(r"보개장면")
    while idx < len(input_text):
        match = pattern.search(input_text, idx)
        if not match:
            out.append(input_text[idx:])
            break
        start = match.start()
        out.append(input_text[idx:start])
        brace_pos = input_text.find("{", match.end())
        if brace_pos == -1:
            out.append(input_text[start:match.end()])
            idx = match.end()
            continue
        depth = 0
        i = brace_pos
        while i < len(input_text):
            ch = input_text[i]
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    i += 1
                    break
            i += 1
        if depth != 0:
            out.append(input_text[start:])
            break
        j = i
        while j < len(input_text) and input_text[j].isspace():
            j += 1
        if j < len(input_text) and input_text[j] == ".":
            j += 1
        idx = j
        removed += 1
    return ("".join(out), removed)


def normalize_inline_statements(input_text: str) -> str:
    out: list[str] = []
    in_string = False
    in_line_comment = False
    escape = False
    i = 0
    while i < len(input_text):
        ch = input_text[i]
        if in_line_comment:
            out.append(ch)
            if ch == "\n":
                in_line_comment = False
            i += 1
            continue
        if in_string:
            out.append(ch)
            if escape:
                escape = False
            elif ch == "\\":
                escape = True
            elif ch == '"':
                in_string = False
            i += 1
            continue
        if ch == "/" and i + 1 < len(input_text) and input_text[i + 1] == "/":
            in_line_comment = True
            out.append(ch)
            out.append(input_text[i + 1])
            i += 2
            continue
        if ch == '"':
            in_string = True
            out.append(ch)
            i += 1
            continue
        if ch == "." and i + 1 < len(input_text) and input_text[i + 1] in (" ", "\t"):
            out.append(".")
            i += 1
            while i < len(input_text) and input_text[i] in (" ", "\t"):
                i += 1
            out.append("\n")
            continue
        out.append(ch)
        i += 1
    return "".join(out)


def normalize_inline_calls(input_text: str) -> str:
    funcs = ("sin", "cos", "tan", "sqrt", "cbrt", "log")
    parts = re.split(r'("(?:(?:\\.)|[^"\\])*")', input_text)
    for idx in range(0, len(parts), 2):
        segment = parts[idx]
        for func in funcs:
            segment = re.sub(
                rf"\b{func}\(([^()]+)\)",
                rf"(\1) {func}",
                segment,
            )
        parts[idx] = segment
    return "".join(parts)


def normalize_pow_half(input_text: str) -> str:
    text = re.sub(r"\)\s*\^\s*0\.5", ") sqrt", input_text)
    text = re.sub(r"([A-Za-z0-9_가-힣\.]+)\s*\^\s*0\.5", r"\1 sqrt", text)
    return text


def flatten_storage_blocks(input_text: str) -> str:
    lines = input_text.splitlines()
    out: list[str] = []
    in_block = False
    depth = 0
    block_head = re.compile(r"^\s*(?:붙박이마련|그릇채비|채비)\s*:\s*\{\s*$")
    decl = re.compile(
        r"^\s*([A-Za-z_가-힣][A-Za-z0-9_가-힣]*)\s*:\s*[^.=<>]+(?:<-|=)\s*(.+?)\s*\.\s*(?://.*)?$"
    )

    for line in lines:
        stripped = line.strip()
        if not in_block and block_head.match(line):
            in_block = True
            depth = 1
            continue
        if not in_block:
            out.append(line)
            continue

        opens = line.count("{")
        closes = line.count("}")
        if opens:
            depth += opens
        if closes:
            depth -= closes
            if depth <= 0:
                in_block = False
                depth = 0
                continue

        if not stripped or stripped.startswith("//"):
            continue
        match = decl.match(line)
        if not match:
            continue
        name = match.group(1)
        expr = match.group(2).strip()
        out.append(f"{name} <- {expr}.")
    return "\n".join(out)


def strip_draw_blocks(input_text: str) -> str:
    lines = input_text.splitlines()
    out: list[str] = []
    skip_list = False
    for line in lines:
        stripped = line.strip()
        if skip_list:
            if "]." in stripped:
                skip_list = False
            continue
        if "그림 <-" in stripped and "[" in stripped:
            skip_list = True
            continue
        if "그림 보여주기" in stripped:
            continue
        if "값꾸러미" in stripped and re.search(r"\b(pivot|bob|그림)\b", stripped):
            continue
        out.append(line)
    return "\n".join(out)


def normalize_int_cast(input_text: str) -> str:
    return re.sub(r"\)\s*정수로\.", ").", input_text)


def rewrite_legacy_formula_assignments(input_text: str) -> str:
    """legacy `a = (#ascii1) 수식{...}` 를 strict 대입(`<-)`으로 변환한다."""
    lines = input_text.splitlines()
    out: list[str] = []
    pattern = re.compile(r"^(\s*[^=\n]+?)\s*=\s*(\(\s*#[^\)]+\)\s*수식\s*\{.*)$")
    for line in lines:
        stripped = line.strip()
        if not stripped or stripped.startswith("#") or stripped.startswith("//"):
            out.append(line)
            continue
        match = pattern.match(line)
        if not match:
            out.append(line)
            continue
        lhs = match.group(1).rstrip()
        rhs = match.group(2).lstrip()
        if "<-" in lhs or "->" in lhs:
            out.append(line)
            continue
        out.append(f"{lhs} <- {rhs}")
    return "\n".join(out)


def rewrite_legacy_boim_blocks(input_text: str) -> str:
    """legacy `보임 { key: expr. }.` 블록을 `expr 보여주기.` 목록으로 변환한다."""
    lines = input_text.splitlines()
    out: list[str] = []
    idx = 0
    header = re.compile(r"^(\s*)보임\s*\{\s*$")
    item = re.compile(r"^[^:]+:\s*(.+)\.\s*$")
    while idx < len(lines):
        line = lines[idx]
        match = header.match(line)
        if not match:
            out.append(line)
            idx += 1
            continue

        indent = match.group(1)
        cursor = idx + 1
        emits: list[str] = []
        convertible = True
        closed = False
        while cursor < len(lines):
            raw = lines[cursor]
            stripped = raw.strip()
            if stripped in ("}.", "}"):
                closed = True
                break
            if not stripped or stripped.startswith("//"):
                cursor += 1
                continue
            row = item.match(stripped)
            if not row:
                convertible = False
                break
            expr = row.group(1).strip()
            emits.append(f"{indent}  {expr} 보여주기.")
            cursor += 1

        if not closed or not convertible:
            out.append(line)
            idx += 1
            continue

        out.extend(emits)
        idx = cursor + 1
    return "\n".join(out)


def _strip_line_comment(line: str) -> str:
    quote = ""
    idx = 0
    while idx < len(line) - 1:
        ch = line[idx]
        if quote:
            if ch == quote and (idx == 0 or line[idx - 1] != "\\"):
                quote = ""
            idx += 1
            continue
        if ch in ("'", '"'):
            quote = ch
            idx += 1
            continue
        if ch == "/" and line[idx + 1] == "/":
            return line[:idx]
        idx += 1
    return line


def _split_top_level_args(text: str) -> list[str]:
    src = str(text or "").strip()
    if not src:
        return []
    out: list[str] = []
    start = 0
    depth = 0
    quote = ""
    for idx, ch in enumerate(src):
        if quote:
            if ch == quote and (idx == 0 or src[idx - 1] != "\\"):
                quote = ""
            continue
        if ch in ("'", '"'):
            quote = ch
            continue
        if ch in ("(", "[", "{"):
            depth += 1
            continue
        if ch in (")", "]", "}"):
            depth = max(0, depth - 1)
            continue
        if ch == "," and depth == 0:
            token = src[start:idx].strip()
            if token:
                out.append(token)
            start = idx + 1
    tail = src[start:].strip()
    if tail:
        out.append(tail)
    return out


def _parse_shape_args(arg_text: str) -> dict:
    positionals: list[str] = []
    named: dict[str, str] = {}
    for token in _split_top_level_args(arg_text):
        if "=" not in token:
            positionals.append(token.strip())
            continue
        key, value = token.split("=", 1)
        key = key.strip()
        value = value.strip()
        if key:
            named[key] = value
        elif value:
            positionals.append(value)
    return {"positionals": positionals, "named": named}


def _pick_shape_arg(args: dict, keys: list[object], fallback: str) -> str:
    positionals = args.get("positionals") or []
    named = args.get("named") or {}
    for key in keys:
        if isinstance(key, int):
            if 0 <= key < len(positionals):
                value = str(positionals[key]).strip()
                if value:
                    return value
            continue
        value = str(named.get(str(key), "")).strip()
        if value:
            return value
    return str(fallback or "").strip()


def _build_shape_show_lines(kind: str, args: dict, indent: str) -> list[str]:
    if kind == "점":
        x = _pick_shape_arg(args, ["x", "cx", 0], "0")
        y = _pick_shape_arg(args, ["y", "cy", 1], "0")
        size = _pick_shape_arg(args, ["크기", "size", "r", 2], "0.05")
        color = _pick_shape_arg(args, ["색", "color"], '"#22c55e"')
        return [
            f'{indent}"space2d.shape" 보여주기.',
            f'{indent}"point" 보여주기.',
            f'{indent}"x" 보여주기.',
            f"{indent}{x} 보여주기.",
            f'{indent}"y" 보여주기.',
            f"{indent}{y} 보여주기.",
            f'{indent}"size" 보여주기.',
            f"{indent}{size} 보여주기.",
            f'{indent}"color" 보여주기.',
            f"{indent}{color} 보여주기.",
        ]

    if kind == "선":
        x1 = _pick_shape_arg(args, ["x1", 0], "0")
        y1 = _pick_shape_arg(args, ["y1", 1], "0")
        x2 = _pick_shape_arg(args, ["x2", 2], "0")
        y2 = _pick_shape_arg(args, ["y2", 3], "0")
        stroke = _pick_shape_arg(args, ["색", "stroke"], '"#9ca3af"')
        width = _pick_shape_arg(args, ["굵기", "width"], "0.02")
        return [
            f'{indent}"space2d.shape" 보여주기.',
            f'{indent}"line" 보여주기.',
            f'{indent}"x1" 보여주기.',
            f"{indent}{x1} 보여주기.",
            f'{indent}"y1" 보여주기.',
            f"{indent}{y1} 보여주기.",
            f'{indent}"x2" 보여주기.',
            f"{indent}{x2} 보여주기.",
            f'{indent}"y2" 보여주기.',
            f"{indent}{y2} 보여주기.",
            f'{indent}"stroke" 보여주기.',
            f"{indent}{stroke} 보여주기.",
            f'{indent}"width" 보여주기.',
            f"{indent}{width} 보여주기.",
        ]

    x = _pick_shape_arg(args, ["x", "cx", 0], "0")
    y = _pick_shape_arg(args, ["y", "cy", 1], "0")
    r = _pick_shape_arg(args, ["r", "반지름", 2], "0.08")
    fill = _pick_shape_arg(args, ["색", "fill"], '"#38bdf8"')
    stroke = _pick_shape_arg(args, ["선색", "stroke"], '"#0ea5e9"')
    width = _pick_shape_arg(args, ["굵기", "width"], "0.02")
    return [
        f'{indent}"space2d.shape" 보여주기.',
        f'{indent}"circle" 보여주기.',
        f'{indent}"x" 보여주기.',
        f"{indent}{x} 보여주기.",
        f'{indent}"y" 보여주기.',
        f"{indent}{y} 보여주기.",
        f'{indent}"r" 보여주기.',
        f"{indent}{r} 보여주기.",
        f'{indent}"fill" 보여주기.',
        f"{indent}{fill} 보여주기.",
        f'{indent}"stroke" 보여주기.',
        f"{indent}{stroke} 보여주기.",
        f'{indent}"width" 보여주기.',
        f"{indent}{width} 보여주기.",
    ]


def rewrite_bogae_shape_blocks(input_text: str) -> str:
    lines = input_text.splitlines()
    out: list[str] = []
    idx = 0
    header = re.compile(r"^(\s*)(보개|모양)\s*:?\s*\{\s*(//.*)?$")
    item = re.compile(r"^(점|선|원)\s*\((.*)\)\s*\.\s*$")

    while idx < len(lines):
        line = lines[idx]
        match = header.match(line)
        if not match:
            out.append(line)
            idx += 1
            continue

        indent = f"{match.group(1)}  "
        cursor = idx + 1
        primitives: list[tuple[str, dict]] = []
        closed = False
        convertible = True
        while cursor < len(lines):
            raw = lines[cursor]
            stripped = raw.strip()
            if stripped in ("}.", "}"):
                closed = True
                break
            payload = _strip_line_comment(raw).strip()
            if not payload:
                cursor += 1
                continue
            row = item.match(payload)
            if not row:
                convertible = False
                break
            kind = str(row.group(1) or "").strip()
            args = _parse_shape_args(str(row.group(2) or ""))
            primitives.append((kind, args))
            cursor += 1

        if not closed or not convertible or not primitives:
            out.append(line)
            idx += 1
            continue

        emits = [f'{indent}"space2d" 보여주기.']
        for kind, args in primitives:
            emits.extend(_build_shape_show_lines(kind, args, indent))
        out.extend(emits)
        idx = cursor + 1
    return "\n".join(out)


def rewrite_show_object_particle(input_text: str) -> str:
    """`...을 보여주기.` / `...를 보여주기.` 를 `... 보여주기.` 로 정규화한다."""
    lines = input_text.splitlines()
    out: list[str] = []
    pattern = re.compile(r"^(\s*)(.+?)\s*[을를]\s+보여주기\.\s*(//.*)?$")
    for line in lines:
        stripped = line.strip()
        if not stripped or stripped.startswith("#") or stripped.startswith("//"):
            out.append(line)
            continue
        match = pattern.match(line)
        if not match:
            out.append(line)
            continue
        indent = match.group(1) or ""
        expr = (match.group(2) or "").rstrip()
        comment = (match.group(3) or "").strip()
        out.append(f"{indent}{expr} 보여주기.{(' ' + comment) if comment else ''}")
    return "\n".join(out)


def preprocess_ddn_for_teul(input_text: str, strip_draw: bool = True) -> str:
    text, _ = strip_bogae_scene_blocks(input_text)
    text = flatten_storage_blocks(text)
    text = rewrite_legacy_formula_assignments(text)
    text = rewrite_legacy_boim_blocks(text)
    text = rewrite_bogae_shape_blocks(text)
    text = rewrite_show_object_particle(text)
    text = normalize_inline_calls(text)
    text = normalize_pow_half(text)
    if strip_draw:
        text = strip_draw_blocks(text)
    text = normalize_int_cast(text)
    text = normalize_inline_statements(text)
    return text


def main() -> int:
    parser = argparse.ArgumentParser(description="Export seamgrim.graph.v0 from DDN output")
    parser.add_argument("input", help="path to ddn file that prints x,y lines")
    parser.add_argument("output", nargs="?", help="output json path (optional if --output-dir is set)")
    parser.add_argument("--output-dir", dest="output_dir", help="directory to place output json")
    parser.add_argument("--auto-name", action="store_true", help="auto-name output file when using --output-dir")
    parser.add_argument("--label-from-input", action="store_true", help="use #이름 as series label when available")
    parser.add_argument("--label", default="f(x)", help="series label")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[3]
    input_path = (Path(args.input)).resolve()
    output_path = None
    if args.output:
        output_path = Path(args.output).resolve()
    elif args.output_dir:
        output_dir = Path(args.output_dir).resolve()
        stem = input_path.stem
        if args.auto_name:
            stamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
            name = f"{stem}_{stamp}_graph.json"
        else:
            name = f"{stem}_graph.json"
        output_path = output_dir / name
    else:
        raise ValueError("output path required (provide OUTPUT or --output-dir)")

    input_text = input_path.read_text(encoding="utf-8")
    meta = extract_meta(input_text)
    series_labels = extract_series_labels(input_text)
    source_hash = hash_text(normalize_ddn_for_hash(input_text))
    stripped_text = preprocess_ddn_for_teul(input_text, strip_draw=True)
    teul_input_path = input_path
    temp_path: Optional[Path] = None
    if stripped_text != input_text:
        temp = tempfile.NamedTemporaryFile(
            "w",
            suffix=".ddn",
            delete=False,
            encoding="utf-8",
            newline="\n",
        )
        temp.write(stripped_text)
        temp.flush()
        temp.close()
        temp_path = Path(temp.name)
        teul_input_path = temp_path
    try:
        lines = run_teul_cli(root, teul_input_path)
    finally:
        if temp_path and temp_path.exists():
            temp_path.unlink()
    points = parse_points(lines, series_labels)
    label = args.label
    if args.label_from_input:
        if meta.get("name"):
            label = meta["name"]
        elif meta.get("desc"):
            label = meta["desc"]
    graph = build_graph(points, label, source_hash)
    if series_labels:
        graph["meta"]["series_labels"] = series_labels
        graph["series"][0]["id"] = normalize_series_label(series_labels[0])
    if meta.get("name"):
        graph["meta"]["input_name"] = meta["name"]
    if meta.get("desc"):
        graph["meta"]["input_desc"] = meta["desc"]

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(graph, ensure_ascii=False, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    print(f"ok: {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
