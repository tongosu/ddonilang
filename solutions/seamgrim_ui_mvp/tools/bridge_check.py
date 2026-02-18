#!/usr/bin/env python
import json
import subprocess
import sys
import time
from pathlib import Path
from urllib.error import URLError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[3]
TOOLS_DIR = Path(__file__).resolve().parent
SERVER_URL = "http://127.0.0.1:8787"

sys.path.insert(0, str(TOOLS_DIR))
from export_graph import normalize_ddn_for_hash, hash_text, compute_result_hash


def is_server_alive() -> bool:
    try:
        with urlopen(f"{SERVER_URL}/", timeout=0.5) as resp:
            return resp.status == 200
    except Exception:
        return False


def wait_for_server(timeout_sec: float = 5.0) -> bool:
    deadline = time.time() + timeout_sec
    while time.time() < deadline:
        if is_server_alive():
            return True
        time.sleep(0.2)
    return False


def post_run(ddn_text: str) -> dict:
    payload = json.dumps({"ddn_text": ddn_text}).encode("utf-8")
    req = Request(
        f"{SERVER_URL}/api/run",
        data=payload,
        headers={"Content-Type": "application/json"},
    )
    with urlopen(req, timeout=5.0) as resp:
        body = resp.read().decode("utf-8")
    return json.loads(body)

def fetch_index() -> str:
    with urlopen(f"{SERVER_URL}/", timeout=5.0) as resp:
        return resp.read().decode("utf-8")


def main() -> int:
    sample_path = ROOT / "solutions" / "seamgrim_ui_mvp" / "samples" / "01_line_graph_export.ddn"
    ddn_text = sample_path.read_text(encoding="utf-8")

    started_proc = None
    if not is_server_alive():
        started_proc = subprocess.Popen(
            [sys.executable, str(TOOLS_DIR / "bridge_server.py")],
            cwd=ROOT,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if not wait_for_server():
            if started_proc:
                started_proc.terminate()
            print("bridge server failed to start")
            return 1

    try:
        html = fetch_index()
        if "Seamgrim UI MVP" not in html or "id=\"canvas\"" not in html:
            print("bridge ui check failed: missing expected markup")
            return 1
        payload = post_run(ddn_text)
        if not payload.get("ok"):
            print(f"bridge run failed: {payload.get('error')}")
            return 1
        graph = payload.get("graph", {})
        points = graph.get("series", [{}])[0].get("points", [])
        label = graph.get("series", [{}])[0].get("label", "-")
        input_hash = graph.get("meta", {}).get("source_input_hash", "")
        result_hash = graph.get("meta", {}).get("result_hash", "")
        expected_input_hash = f"sha256:{hash_text(normalize_ddn_for_hash(ddn_text))}"
        expected_result_hash = f"sha256:{compute_result_hash(points)}"
        if input_hash != expected_input_hash:
            print("bridge hash check failed: input_hash mismatch")
            return 1
        if result_hash != expected_result_hash:
            print("bridge hash check failed: result_hash mismatch")
            return 1
        print(f"ok: points={len(points)} label={label}")
        return 0
    except URLError as exc:
        print(f"bridge run failed: {exc}")
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
