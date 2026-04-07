#!/usr/bin/env python
import argparse
import json
import os
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import urlparse
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[3]
TOOLS_DIR = Path(__file__).resolve().parent
DEFAULT_SERVER_URL = os.environ.get("DDN_EXEC_SERVER_URL", "http://127.0.0.1:8787").rstrip("/")
SEAMGRIM_PROJECT_ROOT_PREFIX = "/solutions/seamgrim_ui_mvp"
REQUIRED_NEW_SEED_IDS = (
    "bio_sir_transition_visual_seed_v2",
    "econ_tax_shock_supply_demand_seed_v1",
    "econ_inventory_price_feedback_seed_v2",
    "transport_signal_traffic_seed_v1",
    "queue_mm1_seed_v1",
    "physics_projectile_drag_seed_v1",
    "physics_spring_damper_seed_v1",
    "physics_heat_diffusion_seed_v1",
    "ecology_forest_fire_ca_seed_v1",
    "swarm_boids_alignment_seed_v1",
)
REQUIRED_SUBJECT_REPRESENTATIVE = (
    {
        "lesson_id": "rep_econ_supply_demand_tax_v1",
        "subject": "econ",
        "ssot_pack": "edu_seamgrim_rep_econ_supply_demand_tax_v1",
    },
    {
        "lesson_id": "rep_math_function_line_v1",
        "subject": "math",
        "ssot_pack": "edu_seamgrim_rep_math_function_line_v1",
    },
    {
        "lesson_id": "rep_phys_projectile_xy_v1",
        "subject": "physics",
        "ssot_pack": "edu_seamgrim_rep_phys_projectile_xy_v1",
    },
    {
        "lesson_id": "rep_cs_linear_search_timeline_v1",
        "subject": "cs",
        "ssot_pack": "edu_seamgrim_rep_cs_linear_search_timeline_v1",
    },
    {
        "lesson_id": "rep_science_phase_change_timeline_v1",
        "subject": "science",
        "ssot_pack": "edu_seamgrim_rep_science_phase_change_timeline_v1",
    },
)

sys.path.insert(0, str(TOOLS_DIR))
from bridge_check import build_bridge_report


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


def post_run(base_url: str, ddn_text: str, madi: int | None = None) -> dict:
    req: dict[str, object] = {"ddn_text": ddn_text}
    if madi is not None:
        try:
            parsed = int(madi)
            if parsed > 0:
                req["madi"] = parsed
        except Exception:
            pass
    payload = json.dumps(req).encode("utf-8")
    return _post_run_payload(base_url, payload)


def post_run_json_bom(base_url: str, ddn_text: str, madi: int | None = None) -> dict:
    req: dict[str, object] = {"ddn_text": ddn_text}
    if madi is not None:
        try:
            parsed = int(madi)
            if parsed > 0:
                req["madi"] = parsed
        except Exception:
            pass
    payload = json.dumps(req).encode("utf-8-sig")
    return _post_run_payload(base_url, payload)


def _post_run_payload(base_url: str, payload: bytes) -> dict:
    last_error: Exception | None = None
    for attempt in range(6):
        req = Request(
            f"{base_url}/api/run",
            data=payload,
            headers={"Content-Type": "application/json"},
        )
        try:
            with urlopen(req, timeout=5.0) as resp:
                body = resp.read().decode("utf-8")
            return json.loads(body)
        except (URLError, TimeoutError, ConnectionResetError, OSError, json.JSONDecodeError) as exc:
            last_error = exc
            if attempt >= 5:
                raise
            if not is_server_alive(base_url):
                wait_for_server(base_url, timeout_sec=1.2)
            time.sleep(min(1.5, 0.25 * (attempt + 1)))
    if last_error is not None:
        raise last_error
    raise RuntimeError("api run failed without detail")


def extract_maegim_control_source(payload: dict) -> str:
    if not isinstance(payload, dict):
        return ""
    runtime = payload.get("runtime")
    if isinstance(runtime, dict):
        value = str(runtime.get("maegim_control_source", "")).strip()
        if value:
            return value
    graph = payload.get("graph")
    if isinstance(graph, dict):
        meta = graph.get("meta")
        if isinstance(meta, dict):
            value = str(meta.get("maegim_control_source", "")).strip()
            if value:
                return value
    return ""


def validate_parallel_sample_hash_consistency(base_url: str, ddn_text: str, *, request_count: int = 12) -> tuple[bool, str]:
    if request_count < 3:
        request_count = 3
    payloads: list[dict] = []
    with ThreadPoolExecutor(max_workers=min(8, request_count)) as executor:
        futures = []
        for idx in range(request_count):
            mode = idx % 3
            if mode == 0:
                futures.append(executor.submit(post_run, base_url, ddn_text))
            elif mode == 1:
                futures.append(executor.submit(post_run, base_url, f"\ufeff{ddn_text}"))
            else:
                futures.append(executor.submit(post_run_json_bom, base_url, ddn_text))
        for future in futures:
            payloads.append(future.result())

    baseline_input_hash = ""
    baseline_result_hash = ""
    baseline_maegim_source = ""
    for idx, payload in enumerate(payloads):
        if not isinstance(payload, dict) or not payload.get("ok"):
            return False, f"parallel_run_api_failed:{idx}"
        graph = payload.get("graph")
        if not isinstance(graph, dict):
            return False, f"parallel_graph_missing:{idx}"
        bridge_report = build_bridge_report(
            graph_doc=graph,
            input_text=ddn_text,
            allow_missing_input_text=False,
        )
        if bridge_report.get("input_hash_match") is False:
            return False, f"parallel_hash_input_mismatch:{idx}"
        if bridge_report.get("result_hash_match") is False:
            return False, f"parallel_hash_result_mismatch:{idx}"
        if not bridge_report.get("ok"):
            return False, f"parallel_bridge_check_failed:{idx}"
        input_hash = str(graph.get("meta", {}).get("source_input_hash", "")).strip()
        result_hash = str(graph.get("meta", {}).get("result_hash", "")).strip()
        if not baseline_input_hash:
            baseline_input_hash = input_hash
        elif input_hash and input_hash != baseline_input_hash:
            return False, f"parallel_input_hash_not_equal:{idx}"
        if not baseline_result_hash:
            baseline_result_hash = result_hash
        elif result_hash and result_hash != baseline_result_hash:
            return False, f"parallel_result_hash_not_equal:{idx}"
        maegim_source = extract_maegim_control_source(payload)
        if maegim_source and maegim_source not in {"canon", "legacy"}:
            return False, f"parallel_maegim_source_invalid:{idx}:{maegim_source}"
        if not baseline_maegim_source:
            baseline_maegim_source = maegim_source
        elif maegim_source and baseline_maegim_source and maegim_source != baseline_maegim_source:
            return False, f"parallel_maegim_source_not_equal:{idx}:{maegim_source}"
    return True, ""


def validate_parallel_maegim_control_fetch(
    base_url: str,
    maegim_control_path: str,
    *,
    request_count: int = 10,
) -> tuple[bool, str]:
    paths = build_catalog_candidate_paths(maegim_control_path)
    if not paths:
        return False, "maegim_path_empty"
    if request_count < 2:
        request_count = 2
    rows: list[tuple[str, dict] | None] = []
    with ThreadPoolExecutor(max_workers=min(8, request_count)) as executor:
        futures = [executor.submit(fetch_first_ok_json, base_url, paths) for _ in range(request_count)]
        for future in futures:
            rows.append(future.result())
    source_tag = ""
    for idx, row in enumerate(rows):
        if not row:
            return False, f"maegim_parallel_fetch_failed:{idx}"
        _, payload = row
        if str(payload.get("schema", "")).strip() != "ddn.maegim_control_plan.v1":
            return False, f"maegim_parallel_schema_invalid:{idx}"
        controls = payload.get("controls")
        if not isinstance(controls, list) or not controls:
            return False, f"maegim_parallel_controls_empty:{idx}"
        observed = str(payload.get("source", "")).strip()
        if not observed:
            return False, f"maegim_parallel_source_missing:{idx}"
        if not source_tag:
            source_tag = observed
        elif observed and source_tag and observed != source_tag:
            return False, f"maegim_parallel_source_mismatch:{idx}"
    return True, ""


def validate_seed_pendulum_payload(payload: dict) -> tuple[bool, str]:
    if not isinstance(payload, dict) or not payload.get("ok"):
        return False, f"run_api_failed:{payload.get('error') if isinstance(payload, dict) else 'invalid_payload'}"
    graph = payload.get("graph")
    if not isinstance(graph, dict):
        return False, "graph_missing"
    series = graph.get("series")
    if not isinstance(series, list) or not series:
        return False, "series_missing"
    first = series[0] if isinstance(series[0], dict) else {}
    points = first.get("points")
    if not isinstance(points, list) or not points:
        return False, "points_missing"

    xy: list[tuple[float, float]] = []
    for row in points:
        if not isinstance(row, dict):
            continue
        try:
            x = float(row.get("x"))
            y = float(row.get("y"))
        except (TypeError, ValueError):
            continue
        xy.append((x, y))

    if len(xy) < 150:
        return False, f"points_too_few:{len(xy)}"
    unique_x = {x for x, _ in xy}
    if len(unique_x) < 80:
        return False, f"unique_x_too_few:{len(unique_x)}"
    min_x = min(unique_x)
    max_x = max(unique_x)
    if min_x > 0.0:
        return False, f"x_min_positive:{min_x}"
    if max_x < 4.0:
        return False, f"x_max_too_small:{max_x}"
    ys = [y for _, y in xy]
    if max(ys) <= 0.0 or min(ys) >= 0.0:
        return False, "y_sign_span_missing"

    return True, ""


def validate_college_thermal_payload(payload: dict) -> tuple[bool, str]:
    if not isinstance(payload, dict) or not payload.get("ok"):
        return False, f"run_api_failed:{payload.get('error') if isinstance(payload, dict) else 'invalid_payload'}"
    table = payload.get("table")
    if not isinstance(table, dict):
        return False, "table_missing"
    columns = table.get("columns")
    rows = table.get("rows")
    if not isinstance(columns, list) or not isinstance(rows, list) or not rows:
        return False, "table_invalid"
    keys = [str(col.get("key", "")).strip() for col in columns if isinstance(col, dict)]
    if keys[:4] != ["t", "y", "celsius", "fahrenheit"]:
        return False, "temperature_columns_invalid"
    first_row = rows[0] if isinstance(rows[0], dict) else {}
    last_row = rows[-1] if isinstance(rows[-1], dict) else {}
    if str(first_row.get("celsius", "")).strip() != "80.0@C":
        return False, "first_celsius_text_invalid"
    if str(first_row.get("fahrenheit", "")).strip() != "176.0@F":
        return False, "first_fahrenheit_text_invalid"
    if str(last_row.get("celsius", "")).strip() != "40.0@C":
        return False, "last_celsius_text_invalid"
    if str(last_row.get("fahrenheit", "")).strip() != "104.0@F":
        return False, "last_fahrenheit_text_invalid"
    for idx, row in enumerate(rows):
        if not isinstance(row, dict):
            return False, f"temperature_row_invalid:{idx}"
        celsius_text = str(row.get("celsius", "")).strip()
        fahrenheit_text = str(row.get("fahrenheit", "")).strip()
        if not celsius_text.endswith("@C"):
            return False, f"temperature_celsius_suffix_invalid:{idx}"
        if not fahrenheit_text.endswith("@F"):
            return False, f"temperature_fahrenheit_suffix_invalid:{idx}"
        try:
            celsius_value = float(celsius_text[:-2])
        except Exception:
            return False, f"temperature_celsius_parse_invalid:{idx}"
        try:
            fahrenheit_value = float(fahrenheit_text[:-2])
        except Exception:
            return False, f"temperature_fahrenheit_parse_invalid:{idx}"
        expected_fahrenheit = celsius_value * 9.0 / 5.0 + 32.0
        if abs(fahrenheit_value - expected_fahrenheit) > 0.11:
            return False, f"temperature_relation_invalid:{idx}"
    graph = payload.get("graph")
    if not isinstance(graph, dict):
        return False, "graph_missing"
    series = graph.get("series")
    if not isinstance(series, list) or not series:
        return False, "graph_series_missing"
    points = series[0].get("points") if isinstance(series[0], dict) else None
    if not isinstance(points, list) or len(points) < 10:
        return False, "graph_points_missing"
    return True, ""


def validate_college_math_linear_payload(payload: dict) -> tuple[bool, str]:
    if not isinstance(payload, dict) or not payload.get("ok"):
        return False, f"run_api_failed:{payload.get('error') if isinstance(payload, dict) else 'invalid_payload'}"
    table = payload.get("table")
    if not isinstance(table, dict):
        return False, "table_missing"
    columns = table.get("columns")
    rows = table.get("rows")
    if not isinstance(columns, list) or not isinstance(rows, list) or not rows:
        return False, "table_invalid"
    keys = [str(col.get("key", "")).strip() for col in columns if isinstance(col, dict)]
    if keys[:3] != ["x", "y", "label"]:
        return False, "linear_columns_invalid"
    first_row = rows[0] if isinstance(rows[0], dict) else {}
    last_row = rows[-1] if isinstance(rows[-1], dict) else {}
    if str(first_row.get("label", "")).strip() != "point(-2,-4)":
        return False, "first_label_invalid"
    if str(last_row.get("label", "")).strip() != "point(4,5)":
        return False, "last_label_invalid"
    graph = payload.get("graph")
    if not isinstance(graph, dict):
        return False, "graph_missing"
    series = graph.get("series")
    if not isinstance(series, list) or not series:
        return False, "graph_series_missing"
    points = series[0].get("points") if isinstance(series[0], dict) else None
    if not isinstance(points, list) or len(points) < 10:
        return False, "graph_points_missing"
    return True, ""


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
        except Exception:
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
    ddn_text = sample_path.read_text(encoding="utf-8-sig")

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

        inventory_loaded = fetch_first_ok_json(
            base_url,
            ["/api/lessons/inventory", "/api/lesson-inventory"],
        )
        if not inventory_loaded:
            print("check=lesson_inventory_api_unreachable detail=/api/lessons/inventory")
            return 1
        _, inventory_payload = inventory_loaded
        inventory_rows = inventory_payload.get("lessons")
        if not isinstance(inventory_rows, list) or not inventory_rows:
            print("check=lesson_inventory_api_invalid detail=lessons list empty")
            return 1
        inventory_ids = {
            str(row.get("id", "")).strip()
            for row in inventory_rows
            if isinstance(row, dict) and str(row.get("id", "")).strip()
        }
        inventory_rows_by_id = {
            str(row.get("id", "")).strip(): row
            for row in inventory_rows
            if isinstance(row, dict) and str(row.get("id", "")).strip()
        }
        missing_new_seed_inventory = [seed_id for seed_id in REQUIRED_NEW_SEED_IDS if seed_id not in inventory_ids]
        if missing_new_seed_inventory:
            print(
                "check=lesson_inventory_api_invalid detail=new_seed_ids_missing:"
                + ",".join(missing_new_seed_inventory[:5])
            )
            return 1
        missing_representative_inventory = [
            item["lesson_id"] for item in REQUIRED_SUBJECT_REPRESENTATIVE if item["lesson_id"] not in inventory_ids
        ]
        if missing_representative_inventory:
            print(
                "check=lesson_inventory_api_invalid detail=representative_ids_missing:"
                + ",".join(missing_representative_inventory[:5])
            )
            return 1
        for item in REQUIRED_SUBJECT_REPRESENTATIVE:
            lesson_id = item["lesson_id"]
            expected_subject = str(item["subject"]).strip()
            row = inventory_rows_by_id.get(lesson_id)
            if not isinstance(row, dict):
                print(f"check=lesson_inventory_api_invalid detail=representative_row_missing:{lesson_id}")
                return 1
            observed_subject = str(row.get("subject", "")).strip()
            if observed_subject != expected_subject:
                print(
                    "check=lesson_inventory_api_invalid detail=representative_subject_mismatch:"
                    f"{lesson_id}:{observed_subject}:{expected_subject}"
                )
                return 1
        pendulum_inventory = next(
            (
                row
                for row in inventory_rows
                if isinstance(row, dict) and str(row.get("id", "")).strip() == "physics_pendulum_seed_v1"
            ),
            None,
        )
        if pendulum_inventory is None:
            print("check=lesson_inventory_api_invalid detail=physics_pendulum_seed_v1 missing")
            return 1
        if int(pendulum_inventory.get("maegim_control_warning_count", 0) or 0) <= 0:
            print("check=lesson_inventory_api_invalid detail=maegim_control_warning_count missing for physics_pendulum_seed_v1")
            return 1
        inventory_warning_codes = pendulum_inventory.get("maegim_control_warning_codes")
        if not isinstance(inventory_warning_codes, list) or "W_LEGACY_RANGE_COMMENT_DEPRECATED" not in [
            str(item).strip() for item in inventory_warning_codes
        ]:
            print("check=lesson_inventory_api_invalid detail=maegim_control_warning_codes missing for physics_pendulum_seed_v1")
            return 1
        inventory_warning_names = pendulum_inventory.get("maegim_control_warning_names")
        if not isinstance(inventory_warning_names, list) or "g" not in [str(item).strip() for item in inventory_warning_names]:
            print("check=lesson_inventory_api_invalid detail=maegim_control_warning_names missing for physics_pendulum_seed_v1")
            return 1
        inventory_warning_examples = pendulum_inventory.get("maegim_control_warning_examples")
        if not isinstance(inventory_warning_examples, list) or "g <- (9.8) 매김 { 범위: 1..20. 간격: 0.1. }." not in [
            str(item).strip() for item in inventory_warning_examples
        ]:
            print("check=lesson_inventory_api_invalid detail=maegim_control_warning_examples missing for physics_pendulum_seed_v1")
            return 1
        default_obs = str(pendulum_inventory.get("default_observation", "")).strip()
        default_obs_x = str(pendulum_inventory.get("default_observation_x", "")).strip()
        if not default_obs:
            print("check=lesson_inventory_api_invalid detail=default_observation missing for physics_pendulum_seed_v1")
            return 1
        if not default_obs_x:
            print("check=lesson_inventory_api_invalid detail=default_observation_x missing for physics_pendulum_seed_v1")
            return 1
        pendulum_maegim_paths = pendulum_inventory.get("maegim_control_path")
        if not isinstance(pendulum_maegim_paths, list) or not any(str(item or "").strip() for item in pendulum_maegim_paths):
            print("check=lesson_inventory_api_invalid detail=maegim_control_path missing for physics_pendulum_seed_v1")
            return 1
        pendulum_maegim_path = next((str(item).strip() for item in pendulum_maegim_paths if str(item).strip()), "")
        pendulum_maegim_loaded = fetch_first_ok_json(base_url, build_catalog_candidate_paths(pendulum_maegim_path))
        if not pendulum_maegim_loaded:
            print(f"check=maegim_control_unreachable detail=path={pendulum_maegim_path}")
            return 1
        _, pendulum_maegim_payload = pendulum_maegim_loaded
        if str(pendulum_maegim_payload.get('schema', '')).strip() != "ddn.maegim_control_plan.v1":
            print("check=maegim_control_invalid detail=schema")
            return 1
        pendulum_controls = pendulum_maegim_payload.get("controls")
        if not isinstance(pendulum_controls, list) or not pendulum_controls:
            print("check=maegim_control_invalid detail=controls empty")
            return 1
        pendulum_warnings = pendulum_maegim_payload.get("warnings")
        if not isinstance(pendulum_warnings, list) or not pendulum_warnings:
            print("check=maegim_control_invalid detail=warnings missing")
            return 1
        if "W_LEGACY_RANGE_COMMENT_DEPRECATED" not in [
            str(item.get("code", "")).strip() for item in pendulum_warnings if isinstance(item, dict)
        ]:
            print("check=maegim_control_invalid detail=legacy warning code missing")
            return 1
        parallel_maegim_ok, parallel_maegim_detail = validate_parallel_maegim_control_fetch(
            base_url,
            pendulum_maegim_path,
            request_count=10,
        )
        if not parallel_maegim_ok:
            print(f"check=maegim_control_parallel_fetch_failed detail={parallel_maegim_detail}")
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
        lesson_rows_by_id = {
            str(row.get("id", "")).strip(): row
            for row in lesson_rows
            if isinstance(row, dict) and str(row.get("id", "")).strip()
        }
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
        for item in REQUIRED_SUBJECT_REPRESENTATIVE:
            lesson_id = item["lesson_id"]
            expected_subject = str(item["subject"]).strip()
            expected_ssot_pack = str(item["ssot_pack"]).strip()
            row = lesson_rows_by_id.get(lesson_id)
            if not isinstance(row, dict):
                print(f"check=lessons_index_invalid detail=representative_missing:{lesson_id}")
                return 1
            observed_subject = str(row.get("subject", "")).strip()
            if observed_subject != expected_subject:
                print(
                    "check=lessons_index_invalid detail=representative_subject_mismatch:"
                    f"{lesson_id}:{observed_subject}:{expected_subject}"
                )
                return 1
            observed_source = str(row.get("source", "")).strip()
            if observed_source != "representative_v1":
                print(
                    "check=lessons_index_invalid detail=representative_source_mismatch:"
                    f"{lesson_id}:{observed_source}:representative_v1"
                )
                return 1
            observed_ssot_pack = str(row.get("ssot_pack", "")).strip()
            if observed_ssot_pack != expected_ssot_pack:
                print(
                    "check=lessons_index_invalid detail=representative_ssot_pack_mismatch:"
                    f"{lesson_id}:{observed_ssot_pack}:{expected_ssot_pack}"
                )
                return 1
            rep_ddn_loaded = fetch_first_ok_text(
                base_url,
                build_catalog_candidate_paths(f"/lessons/{lesson_id}/lesson.ddn"),
            )
            if not rep_ddn_loaded:
                print(f"check=lesson_ddn_unreachable detail=representative:{lesson_id}")
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
        seed_ids = {
            str(row.get("seed_id", "")).strip()
            for row in seed_rows
            if isinstance(row, dict) and str(row.get("seed_id", "")).strip()
        }
        missing_new_seed_manifest = [seed_id for seed_id in REQUIRED_NEW_SEED_IDS if seed_id not in seed_ids]
        if missing_new_seed_manifest:
            print("check=seed_manifest_invalid detail=new_seed_ids_missing:" + ",".join(missing_new_seed_manifest[:5]))
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

        pendulum_seed_row = next(
            (
                row
                for row in seed_rows
                if isinstance(row, dict) and str(row.get("seed_id", "")).strip() == "physics_pendulum_seed_v1"
            ),
            None,
        )
        if pendulum_seed_row is None:
            print("check=seed_manifest_invalid detail=physics_pendulum_seed_v1 missing")
            return 1
        pendulum_ddn_path = str(pendulum_seed_row.get("lesson_ddn", "")).strip()
        if not pendulum_ddn_path:
            print("check=seed_manifest_invalid detail=physics_pendulum_seed_v1 lesson_ddn missing")
            return 1
        pendulum_loaded = fetch_first_ok_text(base_url, build_catalog_candidate_paths(pendulum_ddn_path))
        if not pendulum_loaded:
            print(f"check=seed_pendulum_unreachable detail=path={pendulum_ddn_path}")
            return 1
        _, pendulum_ddn_text, _ = pendulum_loaded
        pendulum_payload = post_run(base_url, pendulum_ddn_text, madi=420)
        pendulum_ok, pendulum_detail = validate_seed_pendulum_payload(pendulum_payload)
        if not pendulum_ok:
            print(f"check=seed_pendulum_run_invalid detail={pendulum_detail}")
            return 1
        maegim_control_payload = pendulum_payload.get("maegim_control")
        if not isinstance(maegim_control_payload, dict):
            print("check=seed_pendulum_run_invalid detail=maegim_control missing")
            return 1
        if str(maegim_control_payload.get("schema", "")).strip() != "ddn.maegim_control_plan.v1":
            print("check=seed_pendulum_run_invalid detail=maegim_control schema")
            return 1
        maegim_controls = maegim_control_payload.get("controls")
        if not isinstance(maegim_controls, list) or not maegim_controls:
            print("check=seed_pendulum_run_invalid detail=maegim_control controls empty")
            return 1
        maegim_warnings = maegim_control_payload.get("warnings")
        if not isinstance(maegim_warnings, list) or not maegim_warnings:
            print("check=seed_pendulum_run_invalid detail=maegim_control warnings empty")
            return 1
        if "W_LEGACY_RANGE_COMMENT_DEPRECATED" not in [
            str(item.get("code", "")).strip() for item in maegim_warnings if isinstance(item, dict)
        ]:
            print("check=seed_pendulum_run_invalid detail=maegim_control warning code")
            return 1
        runtime_summary = pendulum_payload.get("runtime")
        if not isinstance(runtime_summary, dict):
            print("check=seed_pendulum_run_invalid detail=runtime summary missing")
            return 1
        if str(runtime_summary.get("schema", "")).strip() != "seamgrim.runtime_summary.v1":
            print("check=seed_pendulum_run_invalid detail=runtime summary schema")
            return 1
        runtime_control_names = runtime_summary.get("maegim_control_names")
        if not isinstance(runtime_control_names, list) or "g" not in [str(item).strip() for item in runtime_control_names]:
            print("check=seed_pendulum_run_invalid detail=runtime maegim control names")
            return 1
        runtime_control_count = runtime_summary.get("maegim_control_count")
        if int(runtime_control_count) != len(maegim_controls):
            print("check=seed_pendulum_run_invalid detail=runtime maegim control count")
            return 1
        runtime_warning_count = runtime_summary.get("maegim_control_warning_count")
        if int(runtime_warning_count) != len(maegim_warnings):
            print("check=seed_pendulum_run_invalid detail=runtime maegim warning count")
            return 1
        runtime_warning_codes = runtime_summary.get("maegim_control_warning_codes")
        if not isinstance(runtime_warning_codes, list) or "W_LEGACY_RANGE_COMMENT_DEPRECATED" not in [
            str(item).strip() for item in runtime_warning_codes
        ]:
            print("check=seed_pendulum_run_invalid detail=runtime maegim warning codes")
            return 1

        thermal_loaded = fetch_first_ok_text(
            base_url, build_catalog_candidate_paths("/lessons/college_physics_thermal/lesson.ddn")
        )
        if not thermal_loaded:
            print("check=thermal_lesson_unreachable detail=college_physics_thermal")
            return 1
        _, thermal_ddn_text, _ = thermal_loaded
        if "@.1C" not in thermal_ddn_text or "@.1F" not in thermal_ddn_text:
            print("check=thermal_lesson_invalid detail=temperature_format_tokens_missing")
            return 1

        linear_loaded = fetch_first_ok_text(
            base_url, build_catalog_candidate_paths("/lessons/college_math_linear/lesson.ddn")
        )
        if not linear_loaded:
            print("check=linear_lesson_unreachable detail=college_math_linear")
            return 1
        _, linear_ddn_text, _ = linear_loaded
        if 'label <- (x=x, y=y) 글무늬{"point({x},{y})"}.' not in linear_ddn_text:
            print("check=linear_lesson_invalid detail=label_template_missing")
            return 1

        # These run-api validations are independent and dominate this check's wall-clock.
        # Execute them concurrently while preserving the same validation criteria.
        # Thermal/linear lessons may emit one sample per 마디 after stateful conversion,
        # so force sufficient madi to keep first/last row assertions stable.
        with ThreadPoolExecutor(max_workers=5) as executor:
            thermal_future = executor.submit(post_run, base_url, thermal_ddn_text, 64)
            linear_future = executor.submit(post_run, base_url, linear_ddn_text, 64)
            sample_future = executor.submit(post_run, base_url, ddn_text)
            sample_bom_future = executor.submit(post_run, base_url, f"\ufeff{ddn_text}")
            sample_json_bom_future = executor.submit(post_run_json_bom, base_url, ddn_text)
            thermal_payload = thermal_future.result()
            linear_payload = linear_future.result()
            payload = sample_future.result()
            payload_bom = sample_bom_future.result()
            payload_json_bom = sample_json_bom_future.result()

        thermal_ok, thermal_detail = validate_college_thermal_payload(thermal_payload)
        if not thermal_ok:
            print(f"check=thermal_run_invalid detail={thermal_detail}")
            return 1

        linear_ok, linear_detail = validate_college_math_linear_payload(linear_payload)
        if not linear_ok:
            print(f"check=linear_run_invalid detail={linear_detail}")
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

        if not payload.get("ok"):
            print(f"check=run_api_failed detail={payload.get('error')}")
            return 1
        if not payload_bom.get("ok"):
            print(f"check=run_api_bom_failed detail={payload_bom.get('error')}")
            return 1
        if not payload_json_bom.get("ok"):
            print(f"check=run_api_json_bom_failed detail={payload_json_bom.get('error')}")
            return 1
        graph = payload.get("graph", {})
        graph_bom = payload_bom.get("graph", {})
        graph_json_bom = payload_json_bom.get("graph", {})
        maegim_source = extract_maegim_control_source(payload)
        maegim_source_bom = extract_maegim_control_source(payload_bom)
        maegim_source_json_bom = extract_maegim_control_source(payload_json_bom)
        for tag in (maegim_source, maegim_source_bom, maegim_source_json_bom):
            if tag and tag not in {"canon", "legacy"}:
                print(f"check=run_maegim_control_source_invalid detail={tag}")
                return 1
        if maegim_source and maegim_source_bom and maegim_source != maegim_source_bom:
            print("check=run_maegim_control_source_mismatch detail=bom")
            return 1
        if maegim_source and maegim_source_json_bom and maegim_source != maegim_source_json_bom:
            print("check=run_maegim_control_source_mismatch detail=json_bom")
            return 1
        points = graph.get("series", [{}])[0].get("points", [])
        label = graph.get("series", [{}])[0].get("label", "-")
        bridge_report = build_bridge_report(
            graph_doc=graph if isinstance(graph, dict) else None,
            input_text=ddn_text,
            allow_missing_input_text=False,
        )
        if bridge_report.get("input_hash_match") is False:
            print("check=run_hash_input_mismatch detail=source_input_hash")
            return 1
        if bridge_report.get("result_hash_match") is False:
            print("check=run_hash_result_mismatch detail=result_hash")
            return 1
        if not bridge_report.get("ok"):
            detail = ";".join(str(row) for row in bridge_report.get("errors", []))
            print(f"check=run_bridge_check_failed detail={detail}")
            return 1
        bridge_report_bom = build_bridge_report(
            graph_doc=graph_bom if isinstance(graph_bom, dict) else None,
            input_text=ddn_text,
            allow_missing_input_text=False,
        )
        if bridge_report_bom.get("input_hash_match") is False:
            print("check=run_bom_hash_input_mismatch detail=source_input_hash")
            return 1
        if bridge_report_bom.get("result_hash_match") is False:
            print("check=run_bom_hash_result_mismatch detail=result_hash")
            return 1
        if not bridge_report_bom.get("ok"):
            detail = ";".join(str(row) for row in bridge_report_bom.get("errors", []))
            print(f"check=run_bom_bridge_check_failed detail={detail}")
            return 1
        bridge_report_json_bom = build_bridge_report(
            graph_doc=graph_json_bom if isinstance(graph_json_bom, dict) else None,
            input_text=ddn_text,
            allow_missing_input_text=False,
        )
        if bridge_report_json_bom.get("input_hash_match") is False:
            print("check=run_json_bom_hash_input_mismatch detail=source_input_hash")
            return 1
        if bridge_report_json_bom.get("result_hash_match") is False:
            print("check=run_json_bom_hash_result_mismatch detail=result_hash")
            return 1
        if not bridge_report_json_bom.get("ok"):
            detail = ";".join(str(row) for row in bridge_report_json_bom.get("errors", []))
            print(f"check=run_json_bom_bridge_check_failed detail={detail}")
            return 1
        baseline_input_hash = str(graph.get("meta", {}).get("source_input_hash", "")).strip()
        bom_input_hash = str(graph_bom.get("meta", {}).get("source_input_hash", "")).strip()
        json_bom_input_hash = str(graph_json_bom.get("meta", {}).get("source_input_hash", "")).strip()
        if baseline_input_hash and bom_input_hash and baseline_input_hash != bom_input_hash:
            print("check=run_bom_hash_not_equal detail=source_input_hash")
            return 1
        if baseline_input_hash and json_bom_input_hash and baseline_input_hash != json_bom_input_hash:
            print("check=run_json_bom_hash_not_equal detail=source_input_hash")
            return 1
        baseline_result_hash = str(graph.get("meta", {}).get("result_hash", "")).strip()
        bom_result_hash = str(graph_bom.get("meta", {}).get("result_hash", "")).strip()
        json_bom_result_hash = str(graph_json_bom.get("meta", {}).get("result_hash", "")).strip()
        if baseline_result_hash and bom_result_hash and baseline_result_hash != bom_result_hash:
            print("check=run_bom_hash_not_equal detail=result_hash")
            return 1
        if baseline_result_hash and json_bom_result_hash and baseline_result_hash != json_bom_result_hash:
            print("check=run_json_bom_hash_not_equal detail=result_hash")
            return 1
        parallel_ok, parallel_detail = validate_parallel_sample_hash_consistency(base_url, ddn_text, request_count=12)
        if not parallel_ok:
            print(f"check=run_parallel_hash_consistency_failed detail={parallel_detail}")
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
