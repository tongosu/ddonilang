#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


SCHEMA = "ddn.showcase.pendulum_tetris.v1"
ROOT = Path(__file__).resolve().parents[2]
PENDULUM_INPUT = "pack/age2_phys_pendulum_v1/input.ddn"
TETRIS_MINI_INPUT = "pack/showcase_tetris_mini_v1/input.ddn"
TETRIS_FULL_INPUT = "pack/game_maker_tetris_full/input.ddn"
DDN_PREPROCESS_JS = "solutions/seamgrim_ui_mvp/ui/runtime/ddn_preprocess.js"


def run_step(cmd: list[str]) -> tuple[int, str, str, int]:
    started = time.perf_counter()
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    elapsed_ms = int((time.perf_counter() - started) * 1000)
    return int(proc.returncode), proc.stdout or "", proc.stderr or "", elapsed_ms


def make_steps(
    *,
    teul_cli: str,
    mode: str,
    seed: str,
    madi_pendulum: int,
    madi_tetris: int,
    out_root: Path,
    tetris_input: str,
    tetris_step_prefix: str,
    tetris_extra_args: list[str],
) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []

    if mode in {"web", "both"}:
        rows.append(
            {
                "name": "pendulum_web",
                "cmd": [
                    teul_cli,
                    "run",
                    PENDULUM_INPUT,
                    "--seed",
                    seed,
                    "--madi",
                    str(madi_pendulum),
                    "--bogae",
                    "web",
                    "--bogae-live",
                    "--sam-live",
                    "web",
                    "--bogae-out",
                    str((out_root / "pendulum_web").as_posix()),
                    "--no-open",
                ],
            }
        )
        rows.append(
            {
                "name": f"{tetris_step_prefix}_web",
                "cmd": [
                    teul_cli,
                    "run",
                    tetris_input,
                    "--seed",
                    seed,
                    "--madi",
                    str(madi_tetris),
                    "--bogae",
                    "web",
                    "--bogae-live",
                    "--sam-live",
                    "web",
                    "--bogae-out",
                    str((out_root / f"{tetris_step_prefix}_web").as_posix()),
                    "--no-open",
                    *tetris_extra_args,
                ],
            }
        )

    if mode in {"console", "both"}:
        rows.append(
            {
                "name": "pendulum_console",
                "cmd": [
                    teul_cli,
                    "run",
                    PENDULUM_INPUT,
                    "--seed",
                    seed,
                    "--madi",
                    str(madi_pendulum),
                    "--bogae",
                    "console",
                    "--bogae-live",
                    "--sam-live",
                    "console",
                    "--console-grid",
                    "80x30",
                    "--console-panel-cols",
                    "0",
                    "--no-open",
                ],
            }
        )
        rows.append(
            {
                "name": f"{tetris_step_prefix}_console",
                "cmd": [
                    teul_cli,
                    "run",
                    tetris_input,
                    "--seed",
                    seed,
                    "--madi",
                    str(madi_tetris),
                    "--bogae",
                    "console",
                    "--bogae-live",
                    "--sam-live",
                    "console",
                    "--console-grid",
                    "23x25",
                    "--console-cell-aspect",
                    "2:1",
                    "--console-panel-cols",
                    "0",
                    "--no-open",
                    *tetris_extra_args,
                ],
            }
        )

    return rows


def ensure_required_files(*, tetris_input: str, needs_preprocess: bool) -> list[str]:
    required = [
        ROOT / PENDULUM_INPUT,
        ROOT / tetris_input,
    ]
    if needs_preprocess:
        required.append(ROOT / DDN_PREPROCESS_JS)
    missing: list[str] = []
    for path in required:
        if not path.exists():
            missing.append(str(path))
    return missing


def preprocess_ddn_for_showcase(*, node_bin: str, src_rel: str, dst_abs: Path) -> tuple[bool, str]:
    src_abs = (ROOT / src_rel).resolve()
    preprocess_abs = (ROOT / DDN_PREPROCESS_JS).resolve()
    dst_abs.parent.mkdir(parents=True, exist_ok=True)
    script = r"""
const fs = require("fs");
const path = require("path");
const { pathToFileURL } = require("url");
async function main() {
  const [srcPath, dstPath, preprocessPath] = process.argv.slice(1);
  const mod = await import(pathToFileURL(path.resolve(preprocessPath)).href);
  const src = fs.readFileSync(srcPath, "utf8");
  const out = mod.preprocessDdnText(src).bodyText;
  fs.mkdirSync(path.dirname(dstPath), { recursive: true });
  fs.writeFileSync(dstPath, out, "utf8");
  console.log("[showcase] preprocessed " + dstPath);
}
main().catch((err) => {
  console.error(String((err && err.stack) || err));
  process.exit(1);
});
""".strip()
    proc = subprocess.run(
        [
            node_bin,
            "-e",
            script,
            str(src_abs),
            str(dst_abs),
            str(preprocess_abs),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        tail = (proc.stderr or proc.stdout or "").strip()
        return False, tail
    return True, str(dst_abs)


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="진자 + 테트리스 쇼케이스를 한 번에 실행한다."
    )
    parser.add_argument("--teul-cli", default="teul-cli", help="teul-cli 실행 파일")
    parser.add_argument(
        "--mode",
        choices=("web", "console", "both"),
        default="both",
        help="실행 모드",
    )
    parser.add_argument("--seed", default="0x0", help="결정성 seed")
    parser.add_argument(
        "--tetris-profile",
        choices=("mini", "full_preprocessed"),
        default="mini",
        help="테트리스 입력 프로파일",
    )
    parser.add_argument("--madi-pendulum", type=int, default=240, help="진자 실행 마디")
    parser.add_argument("--madi-tetris", type=int, default=240, help="테트리스 실행 마디")
    parser.add_argument("--out-root", default="out/showcase", help="산출물 루트")
    parser.add_argument("--node-bin", default="node", help="DDN 전처리에 사용할 node 실행 파일")
    parser.add_argument("--dry-run", action="store_true", help="명령만 출력하고 실행하지 않음")
    parser.add_argument("--json-out", default="", help="실행 리포트 detjson 경로")
    args = parser.parse_args()

    if args.madi_pendulum <= 0 or args.madi_tetris <= 0:
        print("[showcase] madi must be > 0", file=sys.stderr)
        return 1

    out_root = Path(args.out_root)
    tetris_input = TETRIS_MINI_INPUT
    tetris_step_prefix = "tetris_mini"
    tetris_extra_args: list[str] = []
    tetris_preprocessed_output = ""
    needs_preprocess = args.tetris_profile == "full_preprocessed"
    if needs_preprocess:
        tetris_step_prefix = "tetris_full_preprocessed"
        tetris_extra_args = ["--compat-matic-entry"]
        preprocessed_abs = (ROOT / "build" / "showcase_inputs" / "tetris_full_preprocessed.ddn").resolve()
        tetris_preprocessed_output = str(preprocessed_abs)
        tetris_input = str(preprocessed_abs)

    missing = ensure_required_files(
        tetris_input=TETRIS_FULL_INPUT if needs_preprocess else tetris_input,
        needs_preprocess=needs_preprocess,
    )
    if missing:
        print("[showcase] required file missing", file=sys.stderr)
        for item in missing:
            print(f" - {item}", file=sys.stderr)
        return 1

    if needs_preprocess:
        if not args.dry_run:
            ok, detail = preprocess_ddn_for_showcase(
                node_bin=str(args.node_bin),
                src_rel=TETRIS_FULL_INPUT,
                dst_abs=preprocessed_abs,
            )
            if not ok:
                print("[showcase] preprocess failed", file=sys.stderr)
                if detail:
                    print(detail, file=sys.stderr)
                return 1

    steps = make_steps(
        teul_cli=args.teul_cli,
        mode=args.mode,
        seed=args.seed,
        madi_pendulum=args.madi_pendulum,
        madi_tetris=args.madi_tetris,
        out_root=out_root,
        tetris_input=tetris_input,
        tetris_step_prefix=tetris_step_prefix,
        tetris_extra_args=tetris_extra_args,
    )

    report: dict[str, object] = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": True,
        "dry_run": bool(args.dry_run),
        "mode": args.mode,
        "seed": args.seed,
        "tetris_profile": str(args.tetris_profile),
        "madi_pendulum": int(args.madi_pendulum),
        "madi_tetris": int(args.madi_tetris),
        "out_root": str(out_root.as_posix()),
        "pendulum_input": PENDULUM_INPUT,
        "tetris_input": tetris_input,
        "tetris_input_origin": TETRIS_FULL_INPUT if needs_preprocess else TETRIS_MINI_INPUT,
        "tetris_preprocessed_output": tetris_preprocessed_output,
        "steps": [],
    }

    for row in steps:
        name = str(row["name"])
        cmd = [str(item) for item in row["cmd"]]  # type: ignore[index]
        if args.dry_run:
            print(f"[showcase:dry-run] {name}")
            print("  " + " ".join(cmd))
            report["steps"].append(  # type: ignore[index]
                {
                    "name": name,
                    "cmd": cmd,
                    "status": "dry_run",
                    "ok": True,
                    "returncode": 0,
                    "elapsed_ms": 0,
                }
            )
            continue

        print(f"[showcase] run {name}")
        rc, stdout, stderr, elapsed_ms = run_step(cmd)
        ok = rc == 0
        if not ok:
            report["ok"] = False
        report["steps"].append(  # type: ignore[index]
            {
                "name": name,
                "cmd": cmd,
                "status": "pass" if ok else "fail",
                "ok": ok,
                "returncode": rc,
                "elapsed_ms": elapsed_ms,
                "stdout_tail": (stdout.strip().splitlines()[-10:] if stdout.strip() else []),
                "stderr_tail": (stderr.strip().splitlines()[-10:] if stderr.strip() else []),
            }
        )
        if not ok:
            print(f"[showcase] failed step={name} rc={rc}", file=sys.stderr)
            if stdout.strip():
                print(stdout, end="" if stdout.endswith("\n") else "\n", file=sys.stderr)
            if stderr.strip():
                print(stderr, end="" if stderr.endswith("\n") else "\n", file=sys.stderr)
            break

    if args.json_out.strip():
        write_json(Path(args.json_out).resolve(), report)

    if bool(report["ok"]):
        print(
            "[showcase] ok "
            f"mode={args.mode} out={out_root.as_posix()} "
            "hint_web: python -m http.server 8000 (각 out/showcase/*_web 디렉터리)"
        )
        return 0

    print("[showcase] failed", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
