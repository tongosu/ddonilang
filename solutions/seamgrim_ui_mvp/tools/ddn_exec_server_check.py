#!/usr/bin/env python
import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import urlparse
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[3]
TOOLS_DIR = Path(__file__).resolve().parent
DEFAULT_SERVER_URL = os.environ.get("DDN_EXEC_SERVER_URL", "http://127.0.0.1:8787").rstrip("/")
SEAMGRIM_PROJECT_ROOT_PREFIX = "/solutions/seamgrim_ui_mvp"

sys.path.insert(0, str(TOOLS_DIR))
from export_graph import normalize_ddn_for_hash, hash_text, compute_result_hash


def parse_args(argv: list[str] | None = None):
    parser = argparse.ArgumentParser(description="Check Seamgrim DDN exec server")
    parser.add_argument(
        "--base-url",
        default=DEFAULT_SERVER_URL,
        help="target server base url (default: env DDN_EXEC_SERVER_URL or http://127.0.0.1:8787)",
    )
    parser.add_argument(
        "--timeout-sec",
        type=float,
        default=5.0,
        help="startup wait timeout seconds (default: 5.0)",
    )
    return parser.parse_args(argv)


def is_server_alive(base_url: str) -> bool:
    try:
        with urlopen(f"{base_url}/api/health", timeout=0.5) as resp:
            return resp.status == 200
    except Exception:
        return False


def wait_for_server(base_url: str, timeout_sec: float = 5.0) -> bool:
    deadline = time.time() + timeout_sec
    while time.time() < deadline:
        if is_server_alive(base_url):
            return True
        time.sleep(0.2)
    return False


def post_run(base_url: str, ddn_text: str) -> dict:
    payload = json.dumps({"ddn_text": ddn_text}).encode("utf-8")
    req = Request(
        f"{base_url}/api/run",
        data=payload,
        headers={"Content-Type": "application/json"},
    )
    with urlopen(req, timeout=5.0) as resp:
        body = resp.read().decode("utf-8")
    return json.loads(body)


def fetch_index(base_url: str) -> str:
    with urlopen(f"{base_url}/", timeout=5.0) as resp:
        return resp.read().decode("utf-8")


def normalize_catalog_path(path: str) -> str:
    value = str(path or "").strip()
    if not value:
        return ""
    if value.startswith("./"):
        value = value[2:]
    if not value.startswith("/"):
        value = f"/{value}"
    return value


def build_catalog_candidate_paths(path: str) -> list[str]:
    normalized = normalize_catalog_path(path)
    if not normalized:
        return []
    out: list[str] = []
    seen: set[str] = set()

    def push(value: str):
        if not value or value in seen:
            return
        seen.add(value)
        out.append(value)

    push(normalized)
    if normalized.startswith(f"{SEAMGRIM_PROJECT_ROOT_PREFIX}/"):
        push(normalized[len(SEAMGRIM_PROJECT_ROOT_PREFIX):])
    else:
        push(f"{SEAMGRIM_PROJECT_ROOT_PREFIX}{normalized}")
    return out


def fetch_path_text(base_url: str, path: str) -> tuple[str, str]:
    with urlopen(f"{base_url}{path}", timeout=5.0) as resp:
        body = resp.read().decode("utf-8", errors="replace")
        content_type = str(resp.headers.get("Content-Type", "")).strip().lower()
    return body, content_type


def fetch_first_ok_text(base_url: str, paths: list[str]) -> tuple[str, str, str] | None:
    for path in paths:
        try:
            text, content_type = fetch_path_text(base_url, path)
            return path, text, content_type
        except HTTPError:
            continue
        except URLError:
            continue
    return None


def fetch_first_ok_json(base_url: str, paths: list[str]) -> tuple[str, dict] | None:
    loaded = fetch_first_ok_text(base_url, paths)
    if not loaded:
        return None
    path, text, _ = loaded
    try:
        payload = json.loads(text)
    except json.JSONDecodeError:
        return None
    if not isinstance(payload, dict):
        return None
    return path, payload


def can_autostart_local_server(base_url: str) -> bool:
    parsed = urlparse(base_url)
    host = (parsed.hostname or "").strip().lower()
    return host in {"127.0.0.1", "localhost"}


def resolve_bind_from_base_url(base_url: str) -> tuple[str, int]:
    parsed = urlparse(base_url)
    host = parsed.hostname or "127.0.0.1"
    port = parsed.port or 8787
    return host, port


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    base_url = str(args.base_url or "").rstrip("/")
    timeout_sec = max(1.0, float(args.timeout_sec))
    if not base_url:
        print("base url is required")
        return 1

    sample_path = ROOT / "solutions" / "seamgrim_ui_mvp" / "samples" / "01_line_graph_export.ddn"
    ddn_text = sample_path.read_text(encoding="utf-8")

    started_proc = None
    if not is_server_alive(base_url):
        if can_autostart_local_server(base_url):
            host, port = resolve_bind_from_base_url(base_url)
            started_proc = subprocess.Popen(
                [sys.executable, str(TOOLS_DIR / "ddn_exec_server.py"), "--host", host, "--port", str(port)],
                cwd=ROOT,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            if not wait_for_server(base_url, timeout_sec=timeout_sec):
                if started_proc:
                    started_proc.terminate()
                print("ddn exec server failed to start")
                return 1
        else:
            print(f"ddn exec server is not reachable: {base_url}")
            return 1

    try:
        html = fetch_index(base_url)
        has_title = ("Seamgrim UI MVP" in html) or ("셈그림" in html)
        has_canvas = ("id=\"canvas\"" in html) or ("id=\"canvas-bogae\"" in html)
        if not has_title or not has_canvas:
            print("check=index_markup_missing detail=title_or_canvas")
            return 1

        index_loaded = fetch_first_ok_json(base_url, build_catalog_candidate_paths("/lessons/index.json"))
        if not index_loaded:
            print("check=lessons_index_unreachable detail=/lessons/index.json")
            return 1
        _, index_payload = index_loaded
        lesson_rows = index_payload.get("lessons")
        if not isinstance(lesson_rows, list) or not lesson_rows:
            print("check=lessons_index_invalid detail=lessons list empty")
            return 1
        first_lesson = next(
            (str(row.get("id", "")).strip() for row in lesson_rows if isinstance(row, dict) and str(row.get("id", "")).strip()),
            "",
        )
        if not first_lesson:
            print("check=lessons_index_invalid detail=lesson id missing")
            return 1
        lesson_ddn_loaded = fetch_first_ok_text(
            base_url, build_catalog_candidate_paths(f"/lessons/{first_lesson}/lesson.ddn")
        )
        if not lesson_ddn_loaded:
            print(f"check=lesson_ddn_unreachable detail=lesson_id={first_lesson}")
            return 1

        seed_manifest_loaded = fetch_first_ok_json(
            base_url, build_catalog_candidate_paths("/seed_lessons_v1/seed_manifest.detjson")
        )
        if not seed_manifest_loaded:
            print("check=seed_manifest_unreachable detail=seed_manifest.detjson")
            return 1
        _, seed_manifest = seed_manifest_loaded
        seed_rows = seed_manifest.get("seeds")
        if not isinstance(seed_rows, list) or not seed_rows:
            print("check=seed_manifest_invalid detail=seeds list empty")
            return 1
        seed_ddn_path = next(
            (
                str(row.get("lesson_ddn", "")).strip()
                for row in seed_rows
                if isinstance(row, dict) and str(row.get("lesson_ddn", "")).strip()
            ),
            "",
        )
        if not seed_ddn_path:
            print("check=seed_manifest_invalid detail=lesson_ddn missing")
            return 1
        if not fetch_first_ok_text(base_url, build_catalog_candidate_paths(seed_ddn_path)):
            print(f"check=seed_lesson_unreachable detail=path={seed_ddn_path}")
            return 1

        rewrite_manifest_loaded = fetch_first_ok_json(
            base_url, build_catalog_candidate_paths("/lessons_rewrite_v1/rewrite_manifest.detjson")
        )
        if not rewrite_manifest_loaded:
            print("check=rewrite_manifest_unreachable detail=rewrite_manifest.detjson")
            return 1
        _, rewrite_manifest = rewrite_manifest_loaded
        rewrite_rows = rewrite_manifest.get("generated")
        if not isinstance(rewrite_rows, list) or not rewrite_rows:
            print("check=rewrite_manifest_invalid detail=generated list empty")
            return 1
        rewrite_ddn_path = next(
            (
                str(row.get("generated_lesson_ddn", "")).strip()
                for row in rewrite_rows
                if isinstance(row, dict) and str(row.get("generated_lesson_ddn", "")).strip()
            ),
            "",
        )
        if not rewrite_ddn_path:
            print("check=rewrite_manifest_invalid detail=generated_lesson_ddn missing")
            return 1
        if not fetch_first_ok_text(base_url, build_catalog_candidate_paths(rewrite_ddn_path)):
            print(f"check=rewrite_lesson_unreachable detail=path={rewrite_ddn_path}")
            return 1

        wasm_loaded = fetch_first_ok_text(
            base_url,
            [
                "/wasm/ddonirang_tool_bg.wasm",
                "/solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool_bg.wasm",
            ],
        )
        if not wasm_loaded:
            print("check=wasm_asset_unreachable detail=ddonirang_tool_bg.wasm")
            return 1
        _, _, wasm_content_type = wasm_loaded
        if "application/wasm" not in wasm_content_type:
            print(f"check=wasm_mime_invalid detail=content_type={wasm_content_type or '-'}")
            return 1

        payload = post_run(base_url, ddn_text)
        if not payload.get("ok"):
            print(f"check=run_api_failed detail={payload.get('error')}")
            return 1
        graph = payload.get("graph", {})
        points = graph.get("series", [{}])[0].get("points", [])
        label = graph.get("series", [{}])[0].get("label", "-")
        input_hash = graph.get("meta", {}).get("source_input_hash", "")
        result_hash = graph.get("meta", {}).get("result_hash", "")
        expected_input_hash = f"sha256:{hash_text(normalize_ddn_for_hash(ddn_text))}"
        expected_result_hash = f"sha256:{compute_result_hash(points)}"
        if input_hash != expected_input_hash:
            print("check=run_hash_input_mismatch detail=source_input_hash")
            return 1
        if result_hash != expected_result_hash:
            print("check=run_hash_result_mismatch detail=result_hash")
            return 1
        print(f"ok: points={len(points)} label={label} lesson={first_lesson}")
        return 0
    except URLError as exc:
        print(f"check=network_error detail={exc}")
        return 1
    finally:
        if started_proc:
            started_proc.terminate()
            try:
                started_proc.wait(timeout=2.0)
            except subprocess.TimeoutExpired:
                started_proc.kill()
                started_proc.wait(timeout=2.0)


if __name__ == "__main__":
    raise SystemExit(main())
