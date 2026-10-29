#!/usr/bin/env python
import argparse
import hashlib
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path


DEFAULT_PACKS = [
    "paper1_canonize_agglutinative_ast_v2",
]


def run_case(root: Path, input_path: Path, out_artifacts_dir: Path | None = None) -> subprocess.CompletedProcess[str]:
    cmd = [
        "cargo",
        "run",
        "-q",
        "-p",
        "ddonirang-lang",
        "--example",
        "canon_ast_dump",
        "--",
        str(input_path),
    ]
    if out_artifacts_dir is not None:
        cmd.extend(["--out-artifacts", str(out_artifacts_dir)])
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return proc


def make_case_slug(case_id: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "_", case_id.lower()).strip("_")
    return slug or "case"


def build_standard_golden_paths(pack_dir: Path, case_index: int, case_id: str) -> dict[str, Path]:
    stem = f"K{case_index + 1:03d}_{make_case_slug(case_id)}"
    golden_dir = pack_dir / "golden"
    return {
        "canon": golden_dir / f"{stem}.canon.ddn",
        "ast": golden_dir / f"{stem}.ast.detjson",
        "ast_hash": golden_dir / f"{stem}.ast_hash.txt",
    }


def derive_standard_artifacts(actual: str) -> str:
    try:
        payload = json.loads(actual)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"runner 출력이 JSON이 아닙니다: {exc}") from exc
    canon = payload.get("normalized_n1")
    if not isinstance(canon, str):
        raise RuntimeError("runner 출력에 normalized_n1 문자열이 없습니다")
    return canon if canon.endswith("\n") else f"{canon}\n"


def load_pack_spec(pack_dir: Path) -> dict:
    spec_path = pack_dir / "cases.detjson"
    if not spec_path.exists():
        raise RuntimeError(f"{spec_path}: missing")
    try:
        return json.loads(spec_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"{spec_path}: invalid json: {exc}") from exc


def validate_equivalence_groups(
    pack_dir: Path,
    raw_groups: object,
    success_case_ids: list[str],
) -> tuple[list[list[str]], list[str]]:
    if raw_groups is None:
        return ([success_case_ids] if len(success_case_ids) > 1 else []), []
    if not isinstance(raw_groups, list):
        return [], [f"{pack_dir}: equivalence_groups must be a list"]
    failures: list[str] = []
    groups: list[list[str]] = []
    success_id_set = set(success_case_ids)
    seen: set[str] = set()
    for index, raw_group in enumerate(raw_groups):
        if not isinstance(raw_group, list) or not raw_group:
            failures.append(f"{pack_dir}: equivalence_groups[{index}] must be a non-empty list")
            continue
        group: list[str] = []
        for raw_case_id in raw_group:
            case_id = str(raw_case_id).strip()
            if not case_id:
                failures.append(f"{pack_dir}: equivalence_groups[{index}] has empty case id")
                continue
            if case_id not in success_id_set:
                failures.append(
                    f"{pack_dir}: equivalence_groups[{index}] references non-success case ({case_id})"
                )
                continue
            if case_id in seen:
                failures.append(f"{pack_dir}: equivalence_groups duplicate case id ({case_id})")
                continue
            seen.add(case_id)
            group.append(case_id)
        if group:
            groups.append(group)
    return groups, failures


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_pack_metadata(
    pack_dir: Path,
    expected_paths: set[Path],
    run_log_lines: list[str],
) -> None:
    hash_lines = []
    for path in sorted(expected_paths):
        rel = path.relative_to(pack_dir).as_posix()
        hash_lines.append(f"sha256:{sha256_file(path)}  {rel}")
    (pack_dir / "SHA256SUMS.txt").write_text(
        ("\n".join(hash_lines) + "\n") if hash_lines else "",
        encoding="utf-8",
    )
    (pack_dir / "RUN_LOG.txt").write_text(
        "\n".join(run_log_lines) + "\n",
        encoding="utf-8",
    )


def check_pack(root: Path, pack_dir: Path, update: bool, run_log_lines: list[str]) -> list[str]:
    spec = load_pack_spec(pack_dir)
    schema = str(spec.get("schema", ""))
    if schema != "ddn.canon_ast_pack.v1":
        return [f"{pack_dir}: schema mismatch ({schema})"]

    cases = spec.get("cases")
    if not isinstance(cases, list) or not cases:
        return [f"{pack_dir}: cases missing"]

    failures: list[str] = []
    outputs: dict[str, str] = {}
    success_case_ids: list[str] = []
    expected_paths: set[Path] = set()
    for case_index, case in enumerate(cases):
        if not isinstance(case, dict):
            failures.append(f"{pack_dir}: invalid case row")
            continue
        case_id = str(case.get("id", "")).strip() or "<unknown>"
        input_rel = str(case.get("input", "")).strip()
        mode = str(case.get("mode", "success")).strip() or "success"
        expected_rel = str(case.get("expected", "")).strip()
        expected_error_rel = str(case.get("expected_error", "")).strip()
        expected_error_code = str(case.get("expected_error_code", "")).strip()
        if not input_rel:
            failures.append(f"{pack_dir}:{case_id}: input missing")
            continue
        if mode == "success" and not expected_rel:
            failures.append(f"{pack_dir}:{case_id}: expected missing")
            continue
        if mode == "error" and not expected_error_rel:
            failures.append(f"{pack_dir}:{case_id}: expected_error missing")
            continue
        if mode not in {"success", "error"}:
            failures.append(f"{pack_dir}:{case_id}: unsupported mode ({mode})")
            continue

        input_path = pack_dir / input_rel
        if not input_path.exists():
            failures.append(f"{pack_dir}:{case_id}: input missing ({input_rel})")
            continue

        if mode == "error":
            expected_error_path = pack_dir / expected_error_rel
            expected_paths.add(expected_error_path)
            proc = run_case(root, input_path, None)
            if proc.returncode == 0:
                failures.append(f"{pack_dir}:{case_id}: expected error but runner succeeded")
                continue
            actual_error = proc.stderr or proc.stdout
            if not actual_error:
                failures.append(f"{pack_dir}:{case_id}: runner failed without stderr/stdout")
                continue
            if expected_error_code and expected_error_code not in actual_error:
                failures.append(
                    f"{pack_dir}:{case_id}: expected error code missing ({expected_error_code})"
                )
                continue
            if update:
                expected_error_path.parent.mkdir(parents=True, exist_ok=True)
                expected_error_path.write_text(actual_error, encoding="utf-8")
                continue
            if not expected_error_path.exists():
                failures.append(f"{pack_dir}:{case_id}: expected_error missing ({expected_error_rel})")
                continue
            expected_error = expected_error_path.read_text(encoding="utf-8")
            if expected_error != actual_error:
                failures.append(f"{pack_dir}:{case_id}: expected_error mismatch ({expected_error_rel})")
            continue

        expected_path = pack_dir / expected_rel
        expected_paths.add(expected_path)
        try:
            with tempfile.TemporaryDirectory(prefix="canon_ast_dpack_") as temp_dir_raw:
                temp_dir = Path(temp_dir_raw)
                proc = run_case(root, input_path, temp_dir)
                if proc.returncode != 0:
                    detail = (proc.stderr or proc.stdout or "").strip()
                    raise RuntimeError(f"{input_path}: {detail}")
                actual = proc.stdout
                derive_standard_artifacts(actual)
                temp_canon = temp_dir / "canon.ddn"
                temp_ast = temp_dir / "ast.detjson"
                temp_ast_hash = temp_dir / "ast_hash.txt"
                if not temp_canon.exists() or not temp_ast.exists() or not temp_ast_hash.exists():
                    raise RuntimeError("runner가 표준 golden artifact(canon/ast/hash)를 생성하지 않았습니다")
                actual_ast_hash_text = temp_ast_hash.read_text(encoding="utf-8")
                actual_ast_text = temp_ast.read_text(encoding="utf-8")
                actual_canon_text = temp_canon.read_text(encoding="utf-8")
        except RuntimeError as exc:
            failures.append(f"{pack_dir}:{case_id}: {exc}")
            continue

        outputs[case_id] = actual
        success_case_ids.append(case_id)
        standard_paths = build_standard_golden_paths(pack_dir, case_index, case_id)

        if update:
            expected_path.parent.mkdir(parents=True, exist_ok=True)
            expected_path.write_text(actual, encoding="utf-8")
            standard_paths["canon"].parent.mkdir(parents=True, exist_ok=True)
            standard_paths["canon"].write_text(actual_canon_text, encoding="utf-8")
            standard_paths["ast"].write_text(actual_ast_text, encoding="utf-8")
            standard_paths["ast_hash"].write_text(actual_ast_hash_text, encoding="utf-8")
            continue

        if not expected_path.exists():
            failures.append(f"{pack_dir}:{case_id}: expected missing ({expected_rel})")
            continue

        expected = expected_path.read_text(encoding="utf-8")
        if expected != actual:
            failures.append(f"{pack_dir}:{case_id}: expected mismatch ({expected_rel})")

        if all(path.exists() for path in standard_paths.values()):
            expected_canon = standard_paths["canon"].read_text(encoding="utf-8")
            if expected_canon != actual_canon_text:
                failures.append(
                    f"{pack_dir}:{case_id}: canon golden mismatch ({standard_paths['canon'].relative_to(pack_dir)})"
                )
            expected_ast = standard_paths["ast"].read_text(encoding="utf-8")
            if expected_ast != actual_ast_text:
                failures.append(
                    f"{pack_dir}:{case_id}: ast golden mismatch ({standard_paths['ast'].relative_to(pack_dir)})"
                )
            expected_hash = standard_paths["ast_hash"].read_text(encoding="utf-8")
            if expected_hash != actual_ast_hash_text:
                failures.append(
                    f"{pack_dir}:{case_id}: ast_hash golden mismatch ({standard_paths['ast_hash'].relative_to(pack_dir)})"
                )

    groups, group_failures = validate_equivalence_groups(
        pack_dir, spec.get("equivalence_groups"), success_case_ids
    )
    failures.extend(group_failures)

    for group in groups:
        if len(group) < 2:
            continue
        baseline_case = group[0]
        baseline_text = outputs.get(baseline_case)
        if baseline_text is None:
            continue
        for case_id in group[1:]:
            text = outputs.get(case_id)
            if text is None:
                continue
            if text != baseline_text:
                failures.append(
                    f"{pack_dir}: ast output differs ({baseline_case} != {case_id})"
                )

    if update and not failures:
        write_pack_metadata(pack_dir, expected_paths, run_log_lines)

    return failures


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Run canonical AST D-PACKs (agglutinative order equivalence)"
    )
    ap.add_argument("packs", nargs="*", help="pack names under ./pack")
    ap.add_argument("--update", action="store_true", help="update expected outputs")
    args = ap.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack_root = root / "pack"
    pack_names = args.packs or DEFAULT_PACKS
    run_log_lines = ["python " + " ".join(sys.argv), "canon ast dpack updated"]

    failures: list[str] = []
    for name in pack_names:
        pack_dir = pack_root / name
        if not pack_dir.exists():
            failures.append(f"{pack_dir}: missing pack")
            continue
        failures.extend(check_pack(root, pack_dir, args.update, run_log_lines))

    if failures:
        for row in failures:
            print(row)
        return 1

    if args.update:
        print("canon ast dpack updated")
    else:
        print("canon ast dpack ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
