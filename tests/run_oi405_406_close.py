#!/usr/bin/env python
import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

DEFAULT_PACKS = [
    "bogae_api_catalog_v1_basic",
    "bogae_api_catalog_v1_graph",
    "bogae_api_catalog_v1_game_hud",
    "bogae_api_catalog_v1_errors",
    "dotbogi_ddn_interface_v1_smoke",
    "dotbogi_ddn_interface_v1_event_roundtrip",
    "dotbogi_ddn_interface_v1_write_forbidden",
]


def clip_list(values: list[str], limit: int = 3) -> list[str]:
    if len(values) <= limit:
        return values
    return values[:limit] + [f"... ({len(values) - limit} more)"]


def summarize_failed_cases(pack_row: dict, case_limit: int = 3) -> list[dict]:
    out: list[dict] = []
    cases = pack_row.get("cases")
    if not isinstance(cases, list):
        return out
    for case in cases:
        if not isinstance(case, dict):
            continue
        if bool(case.get("ok", True)):
            continue
        row = {"index": int(case.get("index", -1))}
        expected = case.get("expected")
        got = case.get("got")
        stderr = case.get("stderr")
        if isinstance(expected, list):
            row["expected"] = clip_list([str(item) for item in expected], limit=3)
        if isinstance(got, list):
            row["got"] = clip_list([str(item) for item in got], limit=3)
        if isinstance(stderr, list):
            row["stderr"] = clip_list([str(item) for item in stderr], limit=3)
        issues = case.get("issues")
        if isinstance(issues, list):
            row["issues"] = clip_list([str(item) for item in issues], limit=3)
        out.append(row)
        if len(out) >= case_limit:
            break
    return out


def run_one(root: Path, runner: Path, pack: str) -> subprocess.CompletedProcess:
    cmd = [sys.executable, str(runner), pack]
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def run_all(
    root: Path,
    runner: Path,
    packs: list[str],
    report_out: Path | None = None,
) -> subprocess.CompletedProcess:
    cmd = [sys.executable, str(runner), *packs]
    if report_out is not None:
        cmd += ["--report-out", str(report_out)]
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def load_batch_report(path: Path) -> dict[str, dict]:
    if not path.exists():
        return {}
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}
    rows: dict[str, dict] = {}
    if not isinstance(doc, dict):
        return rows
    for row in doc.get("packs", []):
        if not isinstance(row, dict):
            continue
        name = row.get("pack")
        if isinstance(name, str):
            rows[name] = row
    return rows


def validate_batch_report(path: Path, report_rows: dict[str, dict], packs: list[str]) -> list[str]:
    def _is_int(value: object) -> bool:
        return isinstance(value, int) and not isinstance(value, bool)

    def _is_nonneg_int(value: object) -> bool:
        return _is_int(value) and int(value) >= 0

    errors: list[str] = []
    if not path.exists():
        errors.append(f"batch report missing: {path}")
        return errors
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        errors.append(f"batch report parse failed: {path} ({exc})")
        return errors
    if not isinstance(doc, dict):
        errors.append("batch report root must be object")
        return errors

    if doc.get("schema") != "ddn.pack.golden.report.v1":
        errors.append("batch report schema must be ddn.pack.golden.report.v1")

    pack_rows = doc.get("packs")
    if not isinstance(pack_rows, list):
        errors.append("batch report field 'packs' must be list")
        pack_rows = []

    for field in ("overall_ok",):
        if not isinstance(doc.get(field), bool):
            errors.append(f"batch report field '{field}' must be bool")
    for field in ("failure_count", "elapsed_ms", "updated_pack_files"):
        if not _is_nonneg_int(doc.get(field)):
            errors.append(f"batch report field '{field}' must be int")

    names_in_doc: set[str] = set()
    for idx, row in enumerate(pack_rows, 1):
        if not isinstance(row, dict):
            errors.append(f"batch report packs[{idx}] must be object")
            continue
        name = row.get("pack")
        if not isinstance(name, str) or not name.strip():
            errors.append(f"batch report packs[{idx}].pack must be non-empty string")
            continue
        if name in names_in_doc:
            errors.append(f"batch report duplicate pack row in doc: {name}")
        names_in_doc.add(name)

    seen: set[str] = set()
    for pack_name, row in report_rows.items():
        if pack_name in seen:
            errors.append(f"duplicate pack row: {pack_name}")
            continue
        seen.add(pack_name)
        if not isinstance(row.get("ok"), bool):
            errors.append(f"pack '{pack_name}' field 'ok' must be bool")
        case_count = row.get("case_count")
        if not _is_nonneg_int(case_count):
            errors.append(f"pack '{pack_name}' field 'case_count' must be int")
            case_count = 0
        failed_case_count = row.get("failed_case_count")
        if not _is_nonneg_int(failed_case_count):
            errors.append(f"pack '{pack_name}' field 'failed_case_count' must be int")
            failed_case_count = 0
        if not _is_nonneg_int(row.get("elapsed_ms")):
            errors.append(f"pack '{pack_name}' field 'elapsed_ms' must be int")
        cases = row.get("cases")
        if not isinstance(cases, list):
            errors.append(f"pack '{pack_name}' field 'cases' must be list")
            cases = []
        if not isinstance(row.get("errors"), list):
            errors.append(f"pack '{pack_name}' field 'errors' must be list")

        if int(case_count) != len(cases):
            errors.append(
                f"pack '{pack_name}' case_count mismatch: case_count={case_count} actual={len(cases)}"
            )

        failed_rows = 0
        for case_idx, case in enumerate(cases, 1):
            if not isinstance(case, dict):
                errors.append(f"pack '{pack_name}' cases[{case_idx}] must be object")
                continue
            index = case.get("index")
            if not _is_int(index) or int(index) <= 0:
                errors.append(
                    f"pack '{pack_name}' cases[{case_idx}].index must be positive int"
                )
            if not isinstance(case.get("ok"), bool):
                errors.append(f"pack '{pack_name}' cases[{case_idx}].ok must be bool")
            if not isinstance(case.get("checked_ok"), bool):
                errors.append(
                    f"pack '{pack_name}' cases[{case_idx}].checked_ok must be bool"
                )
            if case.get("ok") is False:
                failed_rows += 1
                for field in ("expected", "got", "stderr", "issues"):
                    if field in case and not isinstance(case.get(field), list):
                        errors.append(
                            f"pack '{pack_name}' cases[{case_idx}].{field} must be list when present"
                        )

        if int(failed_case_count) != failed_rows:
            errors.append(
                f"pack '{pack_name}' failed_case_count mismatch: failed_case_count={failed_case_count} actual={failed_rows}"
            )

        if row.get("ok") is True and int(failed_case_count) != 0:
            errors.append(
                f"pack '{pack_name}' ok=true but failed_case_count={failed_case_count}"
            )

    for pack in packs:
        if pack not in report_rows:
            errors.append(f"batch report missing target pack row: {pack}")
    return errors


def build_failure_digest(rows: list[dict], limit: int = 10) -> list[str]:
    def _first_nonempty(items: object) -> str:
        if not isinstance(items, list):
            return ""
        for item in items:
            text = str(item).strip()
            if text:
                return text
        return ""

    def _clip_text(text: str, limit_chars: int = 120) -> str:
        text = " ".join(text.split())
        if len(text) <= limit_chars:
            return text
        return text[:limit_chars] + "..."

    def _pick_detail(items: object) -> str:
        if not isinstance(items, list):
            return ""
        lines = [str(item).strip() for item in items if str(item).strip()]
        if not lines:
            return ""
        for line in lines:
            if "[FAIL]" in line or "expected=" in line or "E_" in line:
                return line
        return lines[0]

    digest: list[str] = []
    for row in rows:
        if bool(row.get("ok", False)):
            continue
        pack = str(row.get("pack", "unknown"))
        failed_case_count = int(row.get("failed_case_count", 0))
        exit_code = int(row.get("exit_code", 1))
        summary = f"pack={pack} exit={exit_code} failed_cases={failed_case_count}"
        failed_cases = row.get("failed_cases")
        if isinstance(failed_cases, list) and failed_cases:
            first = failed_cases[0]
            if isinstance(first, dict):
                idx = first.get("index")
                if isinstance(idx, int) and idx > 0:
                    summary += f" first_case={idx}"
                detail = _first_nonempty(first.get("issues"))
                if not detail:
                    exp = _first_nonempty(first.get("expected"))
                    got = _first_nonempty(first.get("got"))
                    if exp or got:
                        detail = f"expected={exp} got={got}".strip()
                if not detail:
                    detail = _pick_detail(first.get("stderr"))
                if detail:
                    summary += f" detail={_clip_text(detail)}"
        if "detail=" not in summary:
            detail = _pick_detail(row.get("stderr"))
            if detail:
                summary += f" detail={_clip_text(detail)}"
        digest.append(summary)
        if len(digest) >= limit:
            break
    return digest


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run OI-405/OI-406 close packs and optionally write a JSON report."
    )
    parser.add_argument(
        "--pack",
        action="append",
        default=[],
        help="override target pack (repeatable). default is OI-405/406 close pack set",
    )
    parser.add_argument("--report-out", help="write JSON report to this file")
    parser.add_argument(
        "--pack-report-out",
        help="optional path to keep ddn.pack.golden.report.v1 from batch run",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="require strict validation for ddn.pack.golden.report.v1 schema/fields",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    runner = root / "tests" / "run_pack_golden.py"
    packs = args.pack if args.pack else list(DEFAULT_PACKS)
    batch_report_path = (
        Path(args.pack_report_out)
        if args.pack_report_out
        else root / "build" / ".tmp_oi405_406_batch_report.detjson"
    )
    batch_report_path.parent.mkdir(parents=True, exist_ok=True)
    rows = []
    overall_ok = False

    batch = run_all(root, runner, packs, batch_report_path)
    batch_stdout = [line for line in batch.stdout.splitlines() if line.strip()]
    batch_stderr = [line for line in batch.stderr.splitlines() if line.strip()]
    if batch.stdout:
        print(batch.stdout, end="")
    if batch.stderr:
        print(batch.stderr, end="", file=sys.stderr)

    strict_errors: list[str] = []
    if batch.returncode == 0:
        overall_ok = True
        report_packs = load_batch_report(batch_report_path)
        if args.strict:
            strict_errors.extend(validate_batch_report(batch_report_path, report_packs, packs))
            if strict_errors:
                overall_ok = False
        for pack in packs:
            pack_row = report_packs.get(pack)
            ok = bool(pack_row.get("ok")) if isinstance(pack_row, dict) else True
            if not ok:
                overall_ok = False
            failed_case_count = 0
            case_count = 0
            elapsed_ms = 0
            if isinstance(pack_row, dict):
                failed_case_count = int(pack_row.get("failed_case_count", 0))
                case_count = int(pack_row.get("case_count", 0))
                elapsed_ms = int(pack_row.get("elapsed_ms", 0))
            rows.append(
                {
                    "pack": pack,
                    "ok": ok,
                    "exit_code": 0 if ok else 1,
                    "stdout": batch_stdout,
                    "stderr": batch_stderr,
                    "case_count": case_count,
                    "failed_case_count": failed_case_count,
                    "elapsed_ms": elapsed_ms,
                    "failed_cases": summarize_failed_cases(pack_row) if isinstance(pack_row, dict) else [],
                }
            )
    else:
        report_packs = load_batch_report(batch_report_path)
        if args.strict:
            strict_errors.extend(validate_batch_report(batch_report_path, report_packs, packs))
        rerun_targets = [
            pack
            for pack in packs
            if not isinstance(report_packs.get(pack), dict) or not bool(report_packs[pack].get("ok"))
        ]
        if rerun_targets:
            print(
                f"batch failed; rerun failed packs only: {', '.join(rerun_targets)}",
                file=sys.stderr,
            )
        overall_ok = True
        rerun_map: dict[str, subprocess.CompletedProcess] = {}
        for pack in rerun_targets:
            result = run_one(root, runner, pack)
            rerun_map[pack] = result
            if result.stdout:
                print(result.stdout, end="")
            if result.stderr:
                print(result.stderr, end="", file=sys.stderr)

        for pack in packs:
            if pack in rerun_map:
                result = rerun_map[pack]
                ok = result.returncode == 0
                case_count = 0
                failed_case_count = 0
                elapsed_ms = 0
                pack_row = report_packs.get(pack)
                if isinstance(pack_row, dict):
                    case_count = int(pack_row.get("case_count", 0))
                    failed_case_count = int(pack_row.get("failed_case_count", 0))
                    elapsed_ms = int(pack_row.get("elapsed_ms", 0))
                rows.append(
                    {
                        "pack": pack,
                        "ok": ok,
                        "exit_code": result.returncode,
                        "stdout": [line for line in result.stdout.splitlines() if line.strip()],
                        "stderr": [line for line in result.stderr.splitlines() if line.strip()],
                        "case_count": case_count,
                        "failed_case_count": failed_case_count,
                        "elapsed_ms": elapsed_ms,
                        "failed_cases": summarize_failed_cases(pack_row) if isinstance(pack_row, dict) else [],
                    }
                )
                if not ok:
                    overall_ok = False
                continue
            pack_row = report_packs.get(pack)
            ok = bool(pack_row.get("ok")) if isinstance(pack_row, dict) else False
            case_count = 0
            failed_case_count = 0
            elapsed_ms = 0
            if isinstance(pack_row, dict):
                case_count = int(pack_row.get("case_count", 0))
                failed_case_count = int(pack_row.get("failed_case_count", 0))
                elapsed_ms = int(pack_row.get("elapsed_ms", 0))
            if not ok:
                overall_ok = False
            rows.append(
                {
                    "pack": pack,
                    "ok": ok,
                    "exit_code": 0 if ok else 1,
                    "stdout": batch_stdout,
                    "stderr": batch_stderr,
                    "case_count": case_count,
                    "failed_case_count": failed_case_count,
                    "elapsed_ms": elapsed_ms,
                    "failed_cases": summarize_failed_cases(pack_row) if isinstance(pack_row, dict) else [],
                }
            )

    if args.strict and strict_errors:
        overall_ok = False

    if not args.pack_report_out:
        batch_report_path.unlink(missing_ok=True)

    if args.report_out:
        report_path = Path(args.report_out)
        report_path.parent.mkdir(parents=True, exist_ok=True)
        failure_digest = build_failure_digest(rows)
        report = {
            "schema": "ddn.oi.close_report.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "pack_list": packs,
            "packs": rows,
            "overall_ok": overall_ok,
            "failure_digest": failure_digest,
            "strict": bool(args.strict),
            "strict_errors": strict_errors,
        }
        if args.pack_report_out:
            report["pack_report_path"] = str(Path(args.pack_report_out))
        report_path.write_text(
            json.dumps(report, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
            newline="\n",
        )

    if not overall_ok:
        digest = build_failure_digest(rows)
        if strict_errors:
            print("oi405_406 close strict validation failed", file=sys.stderr)
            for item in strict_errors:
                print(f"  {item}", file=sys.stderr)
        if digest:
            print("oi405_406 close failed", file=sys.stderr)
            for line in digest:
                print(f"  {line}", file=sys.stderr)

    return 0 if overall_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
